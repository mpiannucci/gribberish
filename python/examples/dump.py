import gribberish

def read_file(filename: str) -> bytes:
    with open(filename, 'rb') as f: 
        raw_data = f.read()
        return raw_data

def read_and_dump_grib(filename: str):
    raw_grib_data = read_file(filename)
    if not len(raw_grib_data):
        return
    
    messages = gribberish.parse_grib_messages(raw_grib_data)
    for message in messages:
        print(f'Variable: {message.var_name}')
