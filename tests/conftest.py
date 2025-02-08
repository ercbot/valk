import pytest


def pytest_addoption(parser):
    parser.addoption(
        "--no-pull",
        action="store_true",
        default=False,
        help="Skip pulling docker image and use local version",
    )
    parser.addoption(
        "--save-logs",
        action="store_true",
        default=False,
        help="Save container logs to file",
    )
