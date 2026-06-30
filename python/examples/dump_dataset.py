import argparse
import gribberish


def read_file(filename: str) -> bytes:
    with open(filename, 'rb') as f:
        raw_data = f.read()
        return raw_data


def dump_node(node, path="/", indent=0):
    pad = "  " * indent
    print(f"{pad}{path}")
    print(f"{pad}  coords:    {list(node.get('coords', {}))}")
    print(f"{pad}  data_vars: {list(node.get('data_vars', {}))}")
    for name, child in node.get("groups", {}).items():
        dump_node(child, f"{path.rstrip('/')}/{name}", indent + 1)


if __name__ == '__main__':
    parser = argparse.ArgumentParser('Dump a grib 2 file dataset')
    parser.add_argument('infile', metavar='i', type=str, help='Path to grib 2 file to ')
    args = parser.parse_args()
    input_filename = args.infile

    raw = read_file(input_filename)
    dataset = gribberish.parse_grib_dataset(raw)
    print(dataset["attrs"])
    # Conflicting hypercubes are split into nested groups; a conflict-free file
    # is a single root dataset.
    dump_node(dataset)
