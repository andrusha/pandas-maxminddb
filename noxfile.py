import nox

nox.options.sessions = ["test"]
dev_deps = ["maturin", "pygal", "pygaljs", "pytest", "pytest-benchmark[histogram]", "maxminddb"]
lint_deps = ["autopep8", "black", "isort", "flake8"]


@nox.session
def test(session):
    session.install(*dev_deps)
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
def test_wheel_release(session):
    session.install(*dev_deps)
    session.run_always("rm", "-rf", "target/wheels", external=True)
    session.run_always("maturin", "build", "--release", "--sdist")
    session.run("pip", "download", "-d", "target/pip_deps", ".")
    session.install(
        "pandas_maxminddb",
        "--no-index",
        "--find-links",
        "target/wheels",
        "--find-links",
        "target/pip_deps",
        "--force-reinstall",
    )
    session.run("python", "-c", "import pandas_maxminddb")


@nox.session
def bench(session):
    session.install(*dev_deps)
    session.install(".")
    session.run("pytest", "--benchmark-only", "--benchmark-histogram")


@nox.session(reuse_venv=True)
def lint(session):
    session.install(*lint_deps)
    session.run("black", "--check", ".")
    session.run("flake8", ".")


@nox.session(reuse_venv=True)
def format(session):
    session.install(*lint_deps)
    session.run("isort", ".")
    session.run("autopep8", "--in-place", "--recursive", ".")
    session.run("black", ".")
