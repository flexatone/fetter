

# Guarantee a Locked & Reproducible Environment with Every Python Run

<!--
Guarantee a Locked & Reproducible Environment with Every Python Run
Make Every Python Run Locked & Reproducible
A Locked & Reproducible Environment on Every Python Run
Enforce a Locked & Reproducible Environment on Every Python Run
Enforcing Reproducible Python Environments with fetter
# Stop Running Python Blind: Ensure Package Alignment with Every Python Execution
# Stop Running Python Blind: Ensure a Reproducible Environment with Every Python Execution
# Ensure a Reproducible Environment for Every Python Run
# Make Every Python Execution Predictable and Reproducible
-->


For many, daily use of Python involves executing code in an environment of well-defined dependencies. If collaborating with others, dependencies can change without notice; even if dependencies do not change, it is easy to mistakenly install a package in the wrong environment. When local dependencies are misaligned, bad things can happen: behaviors may change, outputs might differ, or known malware might linger.

For compiled languages, alignment of dependencies in distributed binaries is required for creating reproducible builds. For Python, is it possible to enforce reproducible behavior?

Leveraging Python's support for such initialization intervention, the `fetter` command-line tool can configure an environment's `python` to either warn or exit before running with misaligned dependencies. For example, the following command will configure `fetter` to always validate the environment of the locally active `python3` with `requirements.lock`:

```shell
$ fetter -e python3 site-install --bound requirements.lock
```

Implemented in efficient multi-threaded Rust, performance overhead is insignificant. While project-specific Python "front-end" commands like `uv run` or `poetry run` offer related functionality, `fetter` is run on every execution of `python` itself, and can be used with any type of project.

## Validating Environments

To validate an environment, two things are required: (1) the Python executable for that environment and (2) a manifest of dependencies.

Environments are specified with a path to a Python executable, given with the `--exe` or `-e` parameter. If a virtual environment is active, specifying `-e python3` is sufficient.

Dependencies are specified with a path given to the `--bound` parameter. Most Python projects define direct (explicitly imported) dependencies in a `requirements.txt` or `pyproject.toml` file. As direct dependencies generally require many of their own "transitive" dependencies, a direct dependency listing is insufficient to fully define an environment. For this reason, tools such as `pip-compile`, `pipenv`, `poetry` and `uv` offer solutions for maintaining "lock" files: complete definitions of both direct and transitive dependencies.

Given values for `-e` and `--bound`, `fetter validate` can be used to validate an environment. A fresh virtual environment can be created with Python's built-in `venv` to demonstrate:

```shell
$ python3 -m venv ~/.env-test
$ source ~/.env-test/bin/activate
```

Given a "requirements.txt" file with the entries shown below, `pip install -r` can be used to install all packages:

```shell
{.env-test} $ cat requirements.txt
numpy==2.2.3
requests==2.32.2
fetter==1.7.0
{.env-test} $ pip install -r requirements.txt
```

While only three direct dependencies are defined, seven packages are installed: `requests` adds four transitive dependencies. Providing the virtual environment's Python with `-e python3` and the dependencies with `--bound requirements.txt`, `fetter validate` reports these installed but not defined dependencies as "Unrequired":

```shell
{.env-test} % fetter -e python3 validate --bound requirements.txt
Package                   Dependency  Explain     Sites
certifi-2025.1.31                     Unrequired  ~/.env-test/lib/python3.13/site-packages
charset_normalizer-3.4.1              Unrequired  ~/.env-test/lib/python3.13/site-packages
idna-3.10                             Unrequired  ~/.env-test/lib/python3.13/site-packages
urllib3-2.3.0                         Unrequired  ~/.env-test/lib/python3.13/site-packages
```

While unrequired packages can be accepted with the `--superset` flag, this disables enforcement of a truly locked environment. For example, using `--superset`, validation passes with no failure.

```shell
{.env-test} % fetter -e python3 validate --bound requirements.txt --superset
```

To comprehensively validate an environment, a lock file is required. The `--bound` argument will accept managed lock files from `pip-tools`, `pipenv`, `poetry`, `uv`, and even the proposed [`PEP 751`](https://peps.python.org/pep-0751) format. A simple option is to use `pip freeze`:

```shell
{.env-test} % pip freeze > requirements.lock
```

Validating against "requirements.lock", the `--superset` argument is no longer necessary. Now all dependencies (both direct and transitive) are bound within a fixed definition.

```shell
{.env-test} % fetter -e python3 validate --bound requirements.lock
```

## Automating Validation with `fetter site-install`

While environment validation with `fetter` can be done as needed or with every commit (via `pre-commit`), a truly reproducible, locked environment is only maintainable if validation is done before every Python execution. The `fetter site-install` command installs a special ".pth" file in the target environment's "site-packages" directory that, at the start of Python site initialization, runs a pre-configured `fetter validate` command. By default, if validation fails, a warning is printed. As shown below, the same arguments provided to `validate` are provided to `site-install`:

```shell
{.env-test} $ fetter -e python3 site-install --bound requirements.lock
```

Now, on every execution of Python, the environment is validated. For example, after installing an unrequired package, running `python3` displays a validation failure:

```shell
{.env-test} $ pip3 install typing-extensions
{.env-test} $ python3
Failed: fetter 1.7.0: validate --bound requirements.lock
Package                   Dependency  Explain     Sites
typing_extensions-4.12.2              Unrequired  ~/.env-test/lib/python3.13/site-packages
Python 3.13.1 (main, Dec  3 2024, 17:59:52) [Clang 16.0.0 (clang-1600.0.26.4)] on darwin
Type "help", "copyright", "credits" or "license" for more information.
>>>
```

For even stronger control, if environment validation fails, `fetter` can exit the process with a supplied exit code.

```shell
{.env-test} $ fetter -e python3 site-install --bound requirements.lock exit --code 3
{.env-test} $ python3
Failed: fetter 1.7.0: validate --bound requirements.lock
Package                   Dependency  Explain     Sites
typing_extensions-4.12.2              Unrequired  ~/.env-test/lib/python3.13/site-packages
{.env-test} $
```

To uninstall automatic environment validation, use `fetter site-uninstall`:

```shell
{.env-test} $ fetter -e python3 site-uninstall
```

## Active Environment Locking

Unlike compiled languages, where dependencies are locked at build time and unchangeable at run time, local Python environments are "live": Python will attempt to run regardless of if dependencies have drifted from defined expectations. With `fetter`, dependency alignment can be ensured before every execution.


