import nox

nox.options.sessions = ["test"]


@nox.session
def test(session):
    session.install("-rrequirements-dev.txt")
    session.install("maturin")
    session.run_always("maturin", "develop")
    session.run("pytest", "--benchmark-skip")


@nox.session(python=False)
def test_linux_docker(session):
    # In order to run in Linux environment, while developing on another OS
    import os

    session.run("docker", "build", "-t", "pandas_maxminddb", ".")
    pwd = os.getcwd()
    session.run(
        "docker",
        "run",
        "--rm",
        "-t",
        "-v",
        f"{pwd}:/code",
        "pandas_maxminddb",
        "/root/.cargo/bin/cargo",
        "test",
        "--no-default-features",
    )


@nox.session
def bench(session):
    session.install("-rrequirements-dev.txt")
    session.install(".")
    session.run("pytest", "--benchmark-only", "--benchmark-histogram")


@nox.session(reuse_venv=True)
def lint(session):
    session.install("-rrequirements-lint.txt")
    session.run("black", "--check", ".")
    session.run("flake8", ".")


@nox.session(reuse_venv=True)
def format(session):
    session.install("-rrequirements-lint.txt")
    session.run("isort", ".")
    session.run("autopep8", "--in-place", "--recursive", ".")
    session.run("black", ".")
