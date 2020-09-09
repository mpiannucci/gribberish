import csv
import datetime
import matplotlib.pyplot as plt


def extract_data(filename, variable):
    times = []
    data = []
    time_index = 0
    var_index = -1

    with open(filename, 'r') as f:
        fr = csv.reader(f, delimiter=',')
        for row in fr:
            if var_index < 0:
                var_index = row.index(variable)
                if var_index < 0:
                    return None
                continue

            times.append(datetime.datetime.fromisoformat(row[time_index]))
            data.append(float(row[var_index]))

        return (times, data)

if __name__ == '__main__':
    filename = 'output/ri_wave_data.csv'
    variable = 'HTSGW (m)'

    times, sig_heights = extract_data(filename, variable)
    sig_heights = [h * 3.81 for h in sig_heights]

    plt.plot(times, sig_heights, c='green')
    plt.xlabel('Date')
    plt.ylabel('Significant Wave Height (ft)')
    plt.grid(True)
    plt.title('Wave Watch III Model Data')
    plt.show()

            