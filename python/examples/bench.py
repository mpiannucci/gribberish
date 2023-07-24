import argparse
import time
import eccodes
import gribberish

if __name__ == '__main__':
    parser = argparse.ArgumentParser('Dump a grib 2 file dataset')
    parser.add_argument('infile', metavar='i', type=str, help='Path to grib 2 file to ')
    args = parser.parse_args()
    input_filename = args.infile

    with open(input_filename, 'rb') as f:
        raw_data = f.read()

    mapping = gribberish.parse_grib_mapping(raw_data)

    eccodes_times = []
    # First run with eccodes
    print(f'Processing {len(mapping)} messages with eccodes')
    for _, mapped in mapping.items():
        offset = mapped[1]
        size = mapped[2].message_size
        end = offset + size

        message = eccodes.codes_new_from_message(raw_data[offset:end])
        try:
            start = time.time()
            data = eccodes.codes_get_array(message, "values")
            end = time.time()
            eccodes_times.append(end - start)
        finally:
            eccodes.codes_release(message)

    print(f'Average eccodes time: {(sum(eccodes_times) / len(eccodes_times)) * 1000} ms')

    # Then run with gribberish
    gribberish_times = []
    # First run with eccodes
    print(f'Processing {len(mapping)} messages with gribberish')
    for _, mapped in mapping.items():
        offset = mapped[1]
        size = mapped[2].message_size
        end = offset + size
        try:
            start = time.time()
            data = gribberish.parse_grib_array(raw_data, offset)
            end = time.time()
            gribberish_times.append(end - start)
        except:
            pass
    
    print(f'Average gribberish time: {(sum(gribberish_times) / len(gribberish_times)) * 1000} ms')