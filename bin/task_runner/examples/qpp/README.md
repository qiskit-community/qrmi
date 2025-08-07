# Tools to generate input for task runner from Qiskit Pasqal Provider

## Prerequisites
* Python 3.11 or above


## Install dependencies

```shell-session
pip install -f requirements.txt
```

## Tools

### task_runner_input.py

Generates input file in the correct format using [Pulser](https://github.com/qiskit-community/qiskit-pasqal-provider)


Example:
```bash
python task_runner_input.py
```

Output:
`sequence.json` will be created.
