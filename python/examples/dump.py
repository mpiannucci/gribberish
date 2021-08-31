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


if __name__ == '__main__':
    fname = 'gfswave.20210826.t12z.atlocn.0p16.f000.grib2'
    messages = read_gib_messages(fname)
    print('--------------------------------------------------------')
    print(f'{messages[0].forecast_date}')
    for message in messages:
        raw_data = message.raw_data_array()
        fortyone_seventyone_data = message.data_at_location(lat=41.0, lon=289.0)
        print(f'{message.var_abbrev} ({message.var_name} - {message.units}): ({len(raw_data)} {numpy.nanmin(raw_data)} {numpy.nanmax(raw_data)} {fortyone_seventyone_data})')

        if message.var_abbrev == 'HTSGW':
            data = message.data()
            lats = message.latitudes()
            lons = message.longitudes()
            print(message.region)
            print(lats)
            print(lons)

            plt.imshow(data)
            plt.savefig('HTSGW.png')
            numpy.savetxt('HTSGW.txt', data)

            # print(message.region)
            top_left = message.location_data_indices(41.6, 288.4)
            bottom_right = message.location_data_indices(41.1, 288.8)
            fortyy = message.location_data_indices(41.0, 289.0)
            print(top_left)
            print(bottom_right)
            print(fortyy)
            print(numpy.nanmean(data[top_left[0]:bottom_right[0], top_left[1]:bottom_right[1]]))
            print(data[fortyy[0], fortyy[1]])
            print(message.data_at_location(41.0, 289.0))