# Pasqal Local QRMI - Examples in Python

## Prerequisites

* Rust 1.85.1 or above
* Python 3.11 or 3.12
* [QRMI python package installation](../../../../README.md)
* [Munge](https://dun.github.io/munge/)

## Install dependencies

```shell-session
$ source ~/py311_qrmi_venv/bin/activate
$ pip install -r ../requirements.txt
```

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below.

| Environment variables | Descriptions |
| ---- | ---- |
| {resource_name}_QRMI_URL |  URL of the QPU middleware (e.g. http://localhost:4207) |
| QRMI_JOB_UID | ID of the user executing the job |
| QRMI_JOB_ID | ID of the job |


## Create Pulser Sequence file as input

Given a Pulser sequence `sequence`, we can convert it to a JSON string and write it to a file like this:

```python
serialized_sequence = sequence.to_abstract_repr()

with open("pulser_seq.json", "w") as f:
    f.write(serialized_sequence)
```

## How to run

```shell-session
$ python example.py -h
usage: example.py [-h] input

An example of Pasqal Local Python QRMI

positional arguments:
  input       sequence input file

options:
  -h, --help  show this help message and exit
```
For example,
```shell-session
$ python example.py input.json
```
