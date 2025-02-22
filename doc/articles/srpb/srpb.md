
# A Locked & Reproducible Environment on Every Python Run

# Enforce a Locked & Reproducible Environment on Every Python Run

<!--
# Stop Running Python Blind: Ensure Package Alignment with Every Python Execution
# Stop Running Python Blind: Ensure a Reproducible Environment with Every Python Execution
# Ensure a Reproducible Environment for Every Python Run
# Make Every Python Execution Predictable and Reproducible -->

For many, daily use of Python involves writing and executing code in an environment of well-defined dependencies. If collaborating with others, dependencies can change without notice; even if dependencies did not change, it is easy to mistakenly install a package in the wrong environment. When local dependencies are misaligned, bad things can happen: behaviors may change, outputs might differ, or known malware might be linger.

For compiled languages, alignment of dependencies to distributed binaries is required for creating "reproducible builds". For Python, is it possible to enforce reproducible behavior?

Python supports such intervention in initialization, and the `fetter` command-line tool can configure an environment's `python` to either warn or exit before running with misaligned dependencies. Implemented in efficient multi-threaded Rust, performance overhead is insignificant. While project-specific Python "front-end" commands like `uv run` or `poetry run` offer similar functionality, `fetter` is run on every evocation of `python` itself, and can be used with any type of project.

## Validating Environments

Most Python projects define direct (explicitly imported) dependencies in a `requirements.txt` or `pyproject.toml` file. As direct dependencies generally require many "transitive" dependencies, a direct dependency listing is insufficient to fully desribe an environemnt. For this reason tools such as `pip-compile`, `pipenv`, `poetry` and `uv` have offered solutions for creating and maintaing lock files, complete definitions of both direct and transitive dependencies.

## Automating Validation with `fetter site-install`






----------

Unused:

For compiled languages, dependency reproducible builds are required to establish a chain of trust between source code and binaries. Is it possible to have this in Python? While Python runs byte code instead of binaries, is it possible to enforce reproducible behavior by only running Python if the dependencies conform to an explicit definition?


For many, daily use of Python involves writing and executing code in a virtual environment. If collaborating with others, the direct dependencies of this code are documented in a `requirements.txt` or `pyproject.toml` file. If using `uv`, `poetry`, or related tools, a lock file, pinning all direct and transitive dependencies, might also be defined.

The only way to ensure reproducible behavior in Python (as well as reducing the risk of installing malware) is to validate installed virtual environment dependencies against a lock file. If we can do it fast enough, we should do it every time we run Python. That is what the `fetter site-install` command does.

The only way to avoid this in Python is to validate dependencies against a lock file. If we can do it fast enough, we should do it every time we run Python. That is what the `fetter site-install` command does.




===========

uv:

Prior to every uv run invocation, uv will verify that the lockfile is up-to-date with the pyproject.toml, and that the environment is up-to-date with the lockfile, keeping your project in-sync without the need for manual intervention. uv run guarantees that your command is run in a consistent, locked environment.




