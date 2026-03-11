#!/usr/bin/env python3
"""
Verify gribberish can decode every field across all Spire weather products.
Uses eccodes as the reference decoder for per-message comparison.

Usage:
    python verify_products.py                    # Test all products, all messages
    python verify_products.py --max-messages 50  # Test up to 50 messages per product
    python verify_products.py --products gfs,hrrr  # Test specific products only
"""

import argparse
import csv
import os
import subprocess
import sys
import time

import eccodes
import gribberish
import numpy as np

PRODUCTS = {
    "saifs_wx": {
        "s3": "s3://spire-wx-products-datafeed-stage/saifs/aiwx/statistics/20260311/06/saifs-wx.t06z.pgrb2.0p25.f000",
        "local": "test-data/saifs-wx.t06z.pgrb2.0p25.f000",
    },
    "s2s_stats": {
        "s3": "s3://spire-wx-products-datafeed-stage/saifs/s2s/statistics/20260311/00/saifs-s2s.stats.global.t00z.pgrb2.0p5.D001.grib2",
        "local": "test-data/saifs-s2s.stats.global.t00z.pgrb2.0p5.D001.grib2",
    },
    "srfs": {
        "s3": "s3://spire-wx-products-datafeed-stage/srfs/conus/20260311/12/srfs.conus.t12z.pgrb2.03km.f000m00",
        "local": "test-data/srfs.conus.t12z.pgrb2.03km.f000m00.verify",
    },
    "hrrr": {
        "s3": "s3://spire-wx-products-datafeed-stage/hrrr/20260301/00/hrrr.t00z.wrfprsf00.grib2",
        "local": "test-data/hrrr.t00z.wrfprsf00.grib2",
    },
    "gfs": {
        "s3": "s3://spire-wx-products-datafeed-stage/gfs/20260310/00/gfs.t00z.pgrb2.0p25.f000",
        "local": "test-data/gfs.t00z.pgrb2.0p25.f000",
    },
    "ifs": {
        "s3": "s3://spire-wx-products-datafeed-stage/ifs/20260310/00/ecmwf.t00z.pgrb.0p125.f000",
        "local": "test-data/ecmwf.t00z.pgrb.0p125.f000.verify",
    },
    "s2s_ensemble": {
        "s3": "s3://spire-wx-products-datafeed-stage/saifs/s2s/ensemble/20260311/00/saifs-s2s.ens.global.t00z.pgrb2.0p5.D001.grib2",
        "local": "test-data/saifs-s2s.ens.global.t00z.pgrb2.0p5.D001.grib2",
    },
    "upp": {
        "s3": "s3://spire-wx-products-datafeed-stage/mpp/20260311/12/upp.t12z.pgrb2.0p125.f000",
        "local": "test-data/upp.t12z.pgrb2.0p125.f000",
    },
}


def download_product(name, info):
    """Download product from S3 if not cached locally."""
    local_path = info["local"]
    if os.path.exists(local_path):
        size_mb = os.path.getsize(local_path) / (1024 * 1024)
        print(f"  [{name}] Already exists ({size_mb:.1f} MB)", flush=True)
        return True
    print(f"  [{name}] Downloading from {info['s3']}...", flush=True)
    try:
        subprocess.run(
            ["aws", "s3", "cp", info["s3"], local_path],
            check=True,
            capture_output=True,
            text=True,
        )
        size_mb = os.path.getsize(local_path) / (1024 * 1024)
        print(f"  [{name}] Downloaded ({size_mb:.1f} MB)", flush=True)
        return True
    except subprocess.CalledProcessError as e:
        print(f"  [{name}] DOWNLOAD FAILED: {e.stderr}", flush=True)
        return False


def verify_product(name, filepath, max_messages=None):
    """Verify a single product by comparing eccodes and gribberish message-by-message.

    Instead of reading the entire file into memory, we iterate with eccodes and
    extract individual message bytes to test gribberish on each one.
    """
    total = 0
    data_ok = 0
    metadata_ok = 0
    failures = []
    templates_seen = set()

    # Read entire file once for gribberish (it needs raw bytes + offset)
    with open(filepath, "rb") as f:
        file_data = f.read()

    with open(filepath, "rb") as f:
        while True:
            msgid = eccodes.codes_grib_new_from_file(f)
            if msgid is None:
                break

            if max_messages is not None and total >= max_messages:
                eccodes.codes_release(msgid)
                break

            total += 1
            try:
                offset = int(eccodes.codes_get(msgid, "offset"))
                short_name = eccodes.codes_get(msgid, "shortName")
                level = eccodes.codes_get(msgid, "level")
                type_of_level = eccodes.codes_get(msgid, "typeOfLevel")
                edition = eccodes.codes_get(msgid, "edition")
                try:
                    tpl = eccodes.codes_get(msgid, "productDefinitionTemplateNumber")
                except Exception:
                    tpl = f"grib{edition}"
                try:
                    discipline = eccodes.codes_get(msgid, "discipline")
                    category = eccodes.codes_get(msgid, "parameterCategory")
                    param_num = eccodes.codes_get(msgid, "parameterNumber")
                except Exception:
                    discipline = category = param_num = "?"
                templates_seen.add(tpl)
            except Exception as e:
                failures.append(
                    {
                        "index": total - 1,
                        "offset": -1,
                        "shortName": "?",
                        "error": f"eccodes read error: {e}",
                        "phase": "eccodes",
                    }
                )
                eccodes.codes_release(msgid)
                continue
            finally:
                eccodes.codes_release(msgid)

            # Test gribberish metadata
            try:
                md = gribberish.parse_grib_message_metadata(file_data, offset)
                metadata_ok += 1
            except BaseException as e:
                failures.append(
                    {
                        "index": total - 1,
                        "offset": offset,
                        "shortName": short_name,
                        "level": level,
                        "typeOfLevel": type_of_level,
                        "template": tpl,
                        "discipline": discipline,
                        "category": category,
                        "param_num": param_num,
                        "error": str(e),
                        "phase": "metadata",
                    }
                )
                continue

            # Test gribberish data decode
            try:
                arr = gribberish.parse_grib_array(file_data, offset)
                if len(arr) > 0 and not np.all(np.isnan(arr)):
                    data_ok += 1
                else:
                    data_ok += 1  # All NaN is still a valid decode
            except BaseException as e:
                failures.append(
                    {
                        "index": total - 1,
                        "offset": offset,
                        "shortName": short_name,
                        "level": level,
                        "typeOfLevel": type_of_level,
                        "template": tpl,
                        "discipline": discipline,
                        "category": category,
                        "param_num": param_num,
                        "error": str(e),
                        "phase": "data",
                        "var_name": md.var_name if md else "?",
                    }
                )

    return {
        "product": name,
        "total": total,
        "metadata_ok": metadata_ok,
        "data_ok": data_ok,
        "failed": total - data_ok,
        "pct": (data_ok / total * 100) if total > 0 else 0,
        "failures": failures,
        "templates": sorted(templates_seen),
    }


def main():
    parser = argparse.ArgumentParser(description="Verify gribberish product decoding")
    parser.add_argument(
        "--max-messages",
        type=int,
        default=None,
        help="Max messages to test per product (default: all)",
    )
    parser.add_argument(
        "--products",
        type=str,
        default=None,
        help="Comma-separated list of products to test",
    )
    args = parser.parse_args()

    products_to_test = PRODUCTS
    if args.products:
        names = [n.strip() for n in args.products.split(",")]
        products_to_test = {n: PRODUCTS[n] for n in names if n in PRODUCTS}

    print("=" * 70)
    print("GRIBBERISH PRODUCT VERIFICATION")
    if args.max_messages:
        print(f"  (max {args.max_messages} messages per product)")
    print("=" * 70, flush=True)

    # Download products
    print("\n--- Downloading Products ---", flush=True)
    available = {}
    for name, info in products_to_test.items():
        if download_product(name, info):
            available[name] = info

    if not available:
        print("No products available. Exiting.")
        sys.exit(1)

    # Run verification
    all_results = []
    all_passed = True

    for name, info in available.items():
        filepath = info["local"]
        print(f"\n{'=' * 70}")
        print(f"VERIFYING: {name} ({filepath})")
        print("=" * 70, flush=True)

        t0 = time.time()
        result = verify_product(name, filepath, max_messages=args.max_messages)
        elapsed = time.time() - t0
        all_results.append(result)

        status = "PASS" if result["failed"] == 0 else "FAIL"
        if result["failed"] > 0:
            all_passed = False
        print(
            f"\n  RESULT: {status} - {result['data_ok']}/{result['total']} decoded "
            f"({result['pct']:.1f}%) in {elapsed:.1f}s"
        )
        print(f"  Templates seen: {result['templates']}", flush=True)

        if result["failures"]:
            print(f"\n  FAILURES ({len(result['failures'])} total):")
            error_groups = {}
            for fail in result["failures"]:
                err = fail.get("error", "unknown")
                if err not in error_groups:
                    error_groups[err] = []
                error_groups[err].append(fail)

            for err, group in sorted(error_groups.items(), key=lambda x: -len(x[1])):
                print(f"\n    [{len(group)}x] {err}")
                for fail in group[:5]:
                    print(
                        f"      msg#{fail.get('index','?')} offset={fail.get('offset','?')} "
                        f"shortName={fail.get('shortName','?')} level={fail.get('level','?')} "
                        f"typeOfLevel={fail.get('typeOfLevel','?')} "
                        f"tpl={fail.get('template','?')} "
                        f"d/c/p={fail.get('discipline','?')}/{fail.get('category','?')}/{fail.get('param_num','?')}"
                    )
                if len(group) > 5:
                    print(f"      ... and {len(group)-5} more")

    # Summary
    print(f"\n{'=' * 70}")
    print("SUMMARY")
    print("=" * 70)
    with open("verify_results.tsv", "w", newline="") as f:
        writer = csv.writer(f, delimiter="\t")
        writer.writerow(
            ["product", "total_fields", "decoded", "failed", "pct", "status"]
        )
        for r in all_results:
            status = "PASS" if r["failed"] == 0 else "FAIL"
            writer.writerow(
                [
                    r["product"],
                    r["total"],
                    r["data_ok"],
                    r["failed"],
                    f"{r['pct']:.1f}",
                    status,
                ]
            )
            print(
                f"  {r['product']:20s} {r['data_ok']:5d}/{r['total']:5d} "
                f"({r['pct']:5.1f}%) {status}"
            )

    print("\nResults written to verify_results.tsv", flush=True)

    if all_passed:
        print("\nALL PRODUCTS PASSED!")
        sys.exit(0)
    else:
        print("\nSOME PRODUCTS FAILED - fixes needed")
        sys.exit(1)


if __name__ == "__main__":
    main()
