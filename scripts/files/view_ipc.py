import sys

import pyarrow as pa

stream = pa.input_stream(sys.argv[1])
buf = stream.read()

with pa.ipc.open_file(buf) as reader:
    print(reader.read_all())
