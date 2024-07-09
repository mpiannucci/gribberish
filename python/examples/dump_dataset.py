import argparse
import gribberish


def read_file(filename: str) -> bytes:
    with open(filename, 'rb') as f:
        raw_data = f.read()
        return raw_data

if __name__ == '__main__':
    parser = argparse.ArgumentParser('Dump a grib 2 file dataset')
    parser.add_argument('infile', metavar='i', type=str, help='Path to grib 2 file to ')
    args = parser.parse_args()
    input_filename = args.infile

    raw = read_file(input_filename)
    dataset = gribberish.parse_grib_dataset(raw)
    print(dataset["attrs"])
    print(dataset["data_vars"])
    print(dataset["coords"])
