import base64

import pytest
from pytest_httpserver import HTTPServer

from valk.computer import Computer, SystemInfo, ValkAPIError


@pytest.fixture
def mock_system_info():
    return {
        "os_type": "Linux",
        "os_version": "5.15.0",
        "display_width": 1920,
        "display_height": 1080,
    }


def test_init_computer(httpserver: HTTPServer, mock_system_info):
    """Test computer initialization and session creation"""
    # Mock system info endpoint
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    # Mock session creation
    httpserver.expect_request(
        "/v1/session", method="POST", json={"clear_existing": False}
    ).respond_with_json({"session_id": "test-session"})

    computer = Computer(httpserver.url_for("/"))

    assert computer.system_info.os_type == "Linux"
    assert computer.system_info.display_width == 1920
    assert computer._session_id == "test-session"


def test_init_with_existing_session(httpserver: HTTPServer, mock_system_info):
    """Test handling of existing session during initialization"""
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    # Mock session conflict
    httpserver.expect_request(
        "/v1/session", method="POST", json={"clear_existing": False}
    ).respond_with_data(status=409, response_data="Session exists")

    with pytest.raises(ValkAPIError) as exc:
        Computer(httpserver.url_for("/"))
    assert "session is already active" in str(exc.value)


def test_end_session(httpserver: HTTPServer, mock_system_info):
    """Test session cleanup"""
    # Setup initial session
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)
    httpserver.expect_request("/v1/session", method="POST").respond_with_json(
        {"session_id": "test-session"}
    )

    # Mock session deletion
    httpserver.expect_request(
        "/v1/session", method="DELETE", headers={"X-Session-ID": "test-session"}
    ).respond_with_json({})

    computer = Computer(httpserver.url_for("/"))
    computer.end_session()

    assert computer._session_id is None


def test_screenshot(httpserver: HTTPServer, mock_system_info):
    """Test screenshot functionality"""
    # Setup session
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)
    httpserver.expect_request("/v1/session", method="POST").respond_with_json(
        {"session_id": "test-session"}
    )

    # Mock screenshot action
    mock_image = base64.b64encode(b"fake image data").decode()
    httpserver.expect_request(
        "/v1/action",
        method="POST",
        headers={"X-Session-ID": "test-session"},
    ).respond_with_json(
        {
            "id": "response-id",
            "request_id": "request-id",
            "timestamp": "2024-02-14T00:00:00Z",
            "status": "success",
            "action": {"type": "screenshot"},
            "data": {"image": mock_image},
        }
    )

    computer = Computer(httpserver.url_for("/"))
    result = computer.screenshot()
    assert result == mock_image


def test_move_mouse(httpserver: HTTPServer, mock_system_info):
    """Test mouse movement functionality"""
    # Setup session
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)
    httpserver.expect_request("/v1/session", method="POST").respond_with_json(
        {"session_id": "test-session"}
    )

    # Mock mouse move action
    httpserver.expect_request(
        "/v1/action",
        method="POST",
        headers={"X-Session-ID": "test-session"},
    ).respond_with_json(
        {
            "id": "response-id",
            "request_id": "request-id",
            "timestamp": "2024-02-14T00:00:00Z",
            "status": "success",
            "action": {"type": "mouse_move", "input": {"x": 100, "y": 200}},
        }
    )

    computer = Computer(httpserver.url_for("/"))
    computer.move_mouse(100, 200)


def test_invalid_mouse_coordinates(httpserver: HTTPServer, mock_system_info):
    """Test handling of invalid mouse coordinates"""
    # Setup session
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)
    httpserver.expect_request("/v1/session", method="POST").respond_with_json(
        {"session_id": "test-session"}
    )

    computer = Computer(httpserver.url_for("/"))

    with pytest.raises(ValueError) as exc:
        computer.move_mouse(-1, 100)
    assert "X coordinate" in str(exc.value)

    with pytest.raises(ValueError) as exc:
        computer.move_mouse(100, 2000)
    assert "Y coordinate" in str(exc.value)
