
# A Locked & Reproducible Environment on Every Python Run

# Enforce a Locked & Reproducible Environment on Every Python Run

<!--
# Stop Running Python Blind: Ensure Package Alignment with Every Python Execution
# Stop Running Python Blind: Ensure a Reproducible Environment with Every Python Execution
# Ensure a Reproducible Environment for Every Python Run
# Make Every Python Execution Predictable and Reproducible -->

For many, daily use of Python involves executing code in an environment of well-defined dependencies. If collaborating with others, dependencies can change without notice; even if dependencies did not change, it is easy to mistakenly install a package in the wrong environment. When local dependencies are misaligned, bad things can happen: behaviors may change, outputs might differ, or known malware might be linger.

For compiled languages, alignment of dependencies to distributed binaries is required for creating "reproducible builds". For Python, is it possible to enforce reproducible behavior?

Python supports such intervention in initialization: the `fetter` command-line tool can configure an environment's `python` to either warn or exit before running with misaligned dependencies. For example, to always validate the environment of the locally active `python3` with `requirements.lock`, install `fetter` and run the following:

```shell
$ fetter -e python3 site-install --bound requirements.lock
```

Implemented in efficient multi-threaded Rust, performance overhead is insignificant. While project-specific Python "front-end" commands like `uv run` or `poetry run` offer similar functionality, `fetter` is run on every evocation of `python` itself, and can be used with any type of project.

## Validating Environments

To validate an environment, you must specify an environment (via a Python executable) and a manifest of dependencies.

Environments are specified with a relative or absolute path to a Python executable, given with the `--exe` or `-e` parameter. If a virtual environment is active, specifying `python3` is sufficient.

Dependencies are specified with a relative or absolute path given to the `--bound` parameter. Most Python projects define direct (explicitly imported) dependencies in a `requirements.txt` or `pyproject.toml` file. As direct dependencies generally require many of their own "transitive" dependencies, a direct dependency listing is insufficient to fully describe an environment. For this reason tools such as `pip-compile`, `pipenv`, `poetry` and `uv` offer solutions for maintaining "lock" files, complete definitions of both direct and transitive dependencies.

Given values for `-e` and `--bound`, `fetter validate` can be used for ad-hoc environment checks. Lets start by creating and activating a fresh virtual environment:

```shell
$ python3 -m venv ~/.env313-test
$ source ~/.env313-test/bin/activate
```

Given a "requirements.txt" file with three entries, we can install them with `pip`:

```
{.env313-test} $ cat requirements.txt
numpy==2.2.3
requests==2.32.2
fetter==1.6.0
{.env313-test} $ pip install -r requirements.txt
```

As will be clear from the output of the previous command, more than three packages are installed: `requests` adds a number of transitive dependencies. This well be made clear by using `fetter validate` and supplying the current Python with `-e python3` and the dependencies with `--bound requirements.txt`:

```shell
{.env313-test} % fetter -e python3 validate --bound requirements.txt
Package                   Dependency  Explain     Sites
certifi-2025.1.31                     Unrequired  ~/.env313-test/lib/python3.13/site-packages
charset_normalizer-3.4.1              Unrequired  ~/.env313-test/lib/python3.13/site-packages
idna-3.10                             Unrequired  ~/.env313-test/lib/python3.13/site-packages
pip-24.3.1                            Unrequired  ~/.env313-test/lib/python3.13/site-packages
urllib3-2.3.0                         Unrequired  ~/.env313-test/lib/python3.13/site-packages
```

This command fails validation as five "unreqiured" packages (the transitive dependencies) are identified. Unspecified packages can be accepted with the `--superset` argument, though this is undesirable if a locked environment is our goal. When using `--supereset`, no output shows the validation passed.

```shell
{.env313-test} % fetter -e python3 validate --bound requirements.txt --superset
```

To make the environment truly reproducible, a lock file is required. The `--bound` argument will accept lock files from `pip-tools`, `pipenv`, `poetry`, or `uv`. Another option is to simply use `pip freeze`:

```shell
{.env313-test} % pip freeze > requirements.lock
```
Now, the `--superset` argument can be removed and we can unambiguously determine if all dependencies (both direct and transitive) are bound within expectations.

```shell
{.env313-test} % fetter -e python3 validate --bound requirements.txt
```

## Automating Validation with `fetter site-install`

While environment validation can be done as needed, or with every commit via `pre-commit`, a truely deterministics, reprorducable environment is only possible if validation is done before every Python execution. The `fetter site-install` command installs special ".pth" file in the target environment's "site-packages" that, at the start of Python initializaiton, runs `fetter validate`. By default, if validation fails, a warning is printed`. To lock our current environment, run the following command:

```shell
{.env313-test} $ fetter -e python3 site-install --bound requirements.lock
```

Now, on every evocation of this environments Python, the environment will be validated. To see this in action, we can install a package and run `python3`.

```shell
{.env313-test} $ pip3 install typing-extensions
{.env313-test} $ python3
Failed: fetter 1.6.0: validate --bound requirements.lock
Package                   Dependency  Explain     Sites
typing_extensions-4.12.2              Unrequired  ~/.env313-test/lib/python3.13/site-packages
Python 3.13.1 (main, Dec  3 2024, 17:59:52) [Clang 16.0.0 (clang-1600.0.26.4)] on darwin
Type "help", "copyright", "credits" or "license" for more information.
>>>
```

For even stronger control, if environment validation fails, `fetter` can exit the process with supplied exit code.

```shell
{.env313-test} $ fetter -e python3 site-install --bound requirements.lock exit --code 3
{.env313-test} $ python3
Failed: fetter 1.6.0: validate --bound requirements.lock
Package                   Dependency  Explain     Sites
typing_extensions-4.12.2              Unrequired  ~/.env313-test/lib/python3.13/site-packages
{.env313-test} $
```

## Active Environment Locking

Once compiled, the dependencies of binary executable cannot change; a verified, reproducible binary can thus always gauarentee repeatability. Many Python users, on the other hand, run code in a "live" environment, where dependencies can (intentionally or not) be removed, added, or changed at any time. This can lead to a missaligned environment, opening the door to divergent behavior or missing out on upgrades that mitigate vulnerabilities.

With `fetter site-install`, environments can be automatically checked before every Python execution, providing immediate, active awareness of a critical aspect of running Python code: the alignment of dependencies.




<!--
For compiled languages, dependency reproducible builds are required to establish a chain of trust between source code and binaries. Is it possible to have this in Python? While Python runs byte code instead of binaries, is it possible to enforce reproducible behavior by only running Python if the dependencies conform to an explicit definition?


For many, daily use of Python involves writing and executing code in a virtual environment. If collaborating with others, the direct dependencies of this code are documented in a `requirements.txt` or `pyproject.toml` file. If using `uv`, `poetry`, or related tools, a lock file, pinning all direct and transitive dependencies, might also be defined.

The only way to ensure reproducible behavior in Python (as well as reducing the risk of installing malware) is to validate installed virtual environment dependencies against a lock file. If we can do it fast enough, we should do it every time we run Python. That is what the `fetter site-install` command does.

The only way to avoid this in Python is to validate dependencies against a lock file. If we can do it fast enough, we should do it every time we run Python. That is what the `fetter site-install` command does.
 -->

<!--
uv:

Prior to every uv run invocation, uv will verify that the lockfile is up-to-date with the pyproject.toml, and that the environment is up-to-date with the lockfile, keeping your project in-sync without the need for manual intervention. uv run guarantees that your command is run in a consistent, locked environment.


 -->




