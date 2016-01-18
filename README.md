

## Getting ready

### Install capnproto

Spoiler: data format matter. A lot. We'll be experimenting with capn'proto 
among other things, so the code needs it to be there to work. On my mac,
brew version worked fine. On the linux boxes, I had to build and install it,
because the provided version, 0.4, was too old. It is relatively painless, see
https://capnproto.org/install.html . At the time I write this, it gives 
instructions for 0.5.3, and this version works. I have 0.6 on my mac, and
it works too.

### Building zpipe

The deflate2 crate can be frustrating at times. It uses miniz and not the good
old zlib, so sometimes, it just does not work.

The "deflate" files from the original benchmarks are a good example of this.
The zpipe.c is a pipe-able zipper/unzipper for deflate. It's part of zlib,
and it helps when working directly with these files.

Bottom line, you must build it: just run the following command at the top level
of the project.

```shell
cc zpipe.c -O3 -o zpipe -lz
```

### Downloading the data

For the sake of simplicity, I will assume that all data will be at the same
than on my laptop. That's just a data/ directory at the top level of the
project. It can be a symlink or a a plain directory (I use both setups).

Now let's have a look at what the data directory looks like.

```
18/01 16:40 ~/dev/github/dazone% tree -L 3 data
data
├── cap
│   ├── 5nodes
│   │   ├── rankings
│   │   └── uservisits
│   └── tiny
│       └── uservisits
├── cap-gz
│   ├── 1node
│   │   └── uservisits
│   ├── 5nodes
│   │   ├── rankings
│   │   └── uservisits
│   └── tiny
│       └── uservisits
└── text-deflate
    ├── 1node
    │   └── uservisits
    ├── 5nodes
    │   ├── rankings
    │   └── uservisits
    └── tiny
        └── uservisits
```

The first level under data is the format of the files it contains, next comes
what I call the "set": the original benchmark data comes in three sizes: tiny,
1node and 5nodes. I will always use 5nodes for benchmarks, but tiny can be 
convenient for debugging.

Then comes the actual "table": query 2 uses only uservisits.

Now, the downloaded files are the ones in text-deflate.

You'll need working S3 credentials and a setup s3cmd. The version from brew or
apt worked for me. Run s3cmd --configure to provide your credentials, crypt
them if you want to (I prefer to use a readonly S3 IAM account).

Then let's download tiny uservisits:

```
SET=tiny
TABLE=uservisits
mkdir -p data/text-deflate/$SET/$TABLE
s3cmd get --recursive s3://big-data-benchmark/pavlo/text-deflate/$SET/$TABLE/ data/text-deflate/$SET/$TABLE
```

You should see something like: 

```
's3://big-data-benchmark/pavlo/text-deflate/tiny/uservisits/000000_0.deflate' -> 'data/text-deflate/tiny/uservisits/000000_0.deflate'  [1 of 10]
 42347 of 42347   100% in    0s   137.45 kB/s  done
's3://big-data-benchmark/pavlo/text-deflate/tiny/uservisits/000001_0.deflate' -> 'data/text-deflate/tiny/uservisits/000001_0.deflate'  [2 of 10]
 41993 of 41993   100% in    0s   197.36 kB/s  done
's3://big-data-benchmark/pavlo/text-deflate/tiny/uservisits/000002_0.deflate' -> 'data/text-deflate/tiny/uservisits/000002_0.deflate'  [3 of 10]
 41837 of 41837   100% in    0s   185.24 kB/s  done
's3://big-data-benchmark/pavlo/text-deflate/tiny/uservisits/000003_0.deflate' -> 'data/text-deflate/tiny/uservisits/000003_0.deflate'  [4 of 10]
 41194 of 41194   100% in    0s   177.63 kB/s  done
's3://big-data-benchmark/pavlo/text-deflate/tiny/uservisits/000004_0.deflate' -> 'data/text-deflate/tiny/uservisits/000004_0.deflate'  [5 of 10]
 44239 of 44239   100% in    0s   113.19 kB/s  done
's3://big-data-benchmark/pavlo/text-deflate/tiny/uservisits/000005_0.deflate' -> 'data/text-deflate/tiny/uservisits/000005_0.deflate'  [6 of 10]
 42970 of 42970   100% in    0s   198.16 kB/s  done
's3://big-data-benchmark/pavlo/text-deflate/tiny/uservisits/000006_0.deflate' -> 'data/text-deflate/tiny/uservisits/000006_0.deflate'  [7 of 10]
 42364 of 42364   100% in    0s   198.59 kB/s  done
's3://big-data-benchmark/pavlo/text-deflate/tiny/uservisits/000007_0.deflate' -> 'data/text-deflate/tiny/uservisits/000007_0.deflate'  [8 of 10]
 43167 of 43167   100% in    0s    81.08 kB/s  done
's3://big-data-benchmark/pavlo/text-deflate/tiny/uservisits/000008_0.deflate' -> 'data/text-deflate/tiny/uservisits/000008_0.deflate'  [9 of 10]
 40625 of 40625   100% in    0s   196.14 kB/s  done
's3://big-data-benchmark/pavlo/text-deflate/tiny/uservisits/000009_0.deflate' -> 'data/text-deflate/tiny/uservisits/000009_0.deflate'  [10 of 10]
 44494 of 44494   100% in    0s   205.20 kB/s  done
```

If this is working, then good. Now set SET=5nodes and start again. It will take a while. Oh, and 30GB.


