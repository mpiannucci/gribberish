import argparse
import gribberish
import numpy
import matplotlib.pyplot as plt


def read_file(filename: str) -> bytes:
    with open(filename, 'rb') as f:
        raw_data = f.read()
        return raw_data


def read_gib_messages(filename: str):
    raw_grib_data = read_file(filename)
    if not len(raw_grib_data):
        return []
    return gribberish.parse_grib_messages(raw_grib_data)


def create_filename(input_filename: str, message_index: int) -> str:
    return f'{input_filename}.{message_index}.png'


if __name__ == '__main__':
    parser = argparse.ArgumentParser('Dump a grib 2 file to a png raster')
    parser.add_argument('infile', metavar='i', type=str, help='Path to grib 2 file to ')
    args = parser.parse_args()

    input_filename = args.infile

    messages = read_gib_messages(input_filename)
    for index, message in enumerate(messages):
        data = message.data()
        print(message.proj)
        plt.imshow(numpy.ma.masked_where(data < -98, data))
        plt.savefig(create_filename(input_filename, index))
        plt.clf()
