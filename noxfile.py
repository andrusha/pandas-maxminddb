import nox

nox.options.sessions = ["test"]


@nox.session
def test(session):
    session.install("-rrequirements-dev.txt")
    session.install("maturin")
    session.run_always("maturin", "develop")
    session.run("pytest", "--benchmark-skip")


@nox.session
def bench(session):
    session.install("-rrequirements-dev.txt")
    session.install(".")
    session.run("pytest", "--benchmark-only", "--benchmark-histogram")


@nox.session
def lint(session):
    session.install("-rrequirements-lint.txt")
    session.run("black", "--check", ".")
    session.run("flake8", ".")


@nox.session
def format(session):
    session.install("-rrequirements-lint.txt")
    session.run("isort", ".")
    session.run("autopep8", "--in-place", "--recursive", ".")
    session.run("black", ".")
