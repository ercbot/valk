import base64

import pytest
from pytest_httpserver import HTTPServer

from valk.computer import Computer


@pytest.fixture
def mock_system_info():
    return {
        "os_type": "Linux",
        "os_version": "5.15.0",
        "display_width": 1920,
        "display_height": 1080,
    }


def mock_action_response(action: dict, data: dict = None):
    response = {
        "id": "response-id",
        "request_id": "request-id",
        "timestamp": "2024-02-14T00:00:00Z",
        "status": "success",
        "action": action,
    }
    if data:
        response["data"] = data
    return response


def test_init_computer(httpserver: HTTPServer, mock_system_info):
    """Test computer initialization"""
    # Mock system info endpoint
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    computer = Computer(httpserver.url_for("/"))

    assert computer.system_info.os_type == "Linux"
    assert computer.system_info.display_width == 1920


def test_screenshot(httpserver: HTTPServer, mock_system_info):
    """Test screenshot functionality"""
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    # Mock screenshot action
    mock_image = base64.b64encode(b"fake image data").decode()
    httpserver.expect_request(
        "/v1/action",
        method="POST",
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
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    # Mock mouse move action
    httpserver.expect_request(
        "/v1/action",
        method="POST",
    ).respond_with_json(
        mock_action_response({"type": "mouse_move", "input": {"x": 100, "y": 200}})
    )

    computer = Computer(httpserver.url_for("/"))
    computer.move_mouse(100, 200)


def test_left_click(httpserver: HTTPServer, mock_system_info):
    """Test left click functionality"""
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    httpserver.expect_request(
        "/v1/action",
        method="POST",
    ).respond_with_json(mock_action_response({"type": "left_click"}))

    computer = Computer(httpserver.url_for("/"))
    computer.left_click()


def test_right_click(httpserver: HTTPServer, mock_system_info):
    """Test right click functionality"""
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    httpserver.expect_request(
        "/v1/action",
        method="POST",
    ).respond_with_json(mock_action_response({"type": "right_click"}))

    computer = Computer(httpserver.url_for("/"))
    computer.right_click()


def test_middle_click(httpserver: HTTPServer, mock_system_info):
    """Test middle click functionality"""
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    httpserver.expect_request(
        "/v1/action",
        method="POST",
    ).respond_with_json(mock_action_response({"type": "middle_click"}))

    computer = Computer(httpserver.url_for("/"))
    computer.middle_click()


def test_double_click(httpserver: HTTPServer, mock_system_info):
    """Test double click functionality"""
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    httpserver.expect_request(
        "/v1/action",
        method="POST",
    ).respond_with_json(mock_action_response({"type": "double_click"}))

    computer = Computer(httpserver.url_for("/"))
    computer.double_click()


def test_left_click_drag(httpserver: HTTPServer, mock_system_info):
    """Test left click drag functionality"""
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    httpserver.expect_request(
        "/v1/action",
        method="POST",
    ).respond_with_json(
        mock_action_response({"type": "left_click_drag", "input": {"x": 100, "y": 200}})
    )

    computer = Computer(httpserver.url_for("/"))
    computer.left_click_drag(100, 200)


def test_type(httpserver: HTTPServer, mock_system_info):
    """Test type functionality"""
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)
    computer = Computer(httpserver.url_for("/"))

    # Test different unicode characters
    texts = [
        "Hello, world!",
        "こんにちは世界",
        "Привет мир",
        "안녕하세요 세계",
        "🤩🤗🦍🐒🤶",
    ]

    for text in texts:
        httpserver.expect_request(
            "/v1/action",
            method="POST",
        ).respond_with_json(
            mock_action_response({"type": "type", "input": {"text": text}})
        )

        computer.type(text)


def test_key(httpserver: HTTPServer, mock_system_info):
    """Test key functionality"""
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)
    computer = Computer(httpserver.url_for("/"))

    key = ["a", "ctrl+a", "ctrl+shift+a"]

    for k in key:
        httpserver.expect_request(
            "/v1/action",
            method="POST",
        ).respond_with_json(mock_action_response({"type": "key", "input": {"key": k}}))

        computer.key(k)


def test_get_cursor_position(httpserver: HTTPServer, mock_system_info):
    """Test get cursor position functionality"""
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    httpserver.expect_request(
        "/v1/action",
        method="POST",
    ).respond_with_json(
        mock_action_response({"type": "cursor_position"}, {"x": 100, "y": 200})
    )

    computer = Computer(httpserver.url_for("/"))
    position = computer.cursor_position()

    assert position == (100, 200)


def test_invalid_mouse_coordinates(httpserver: HTTPServer, mock_system_info):
    """Test handling of invalid mouse coordinates"""
    httpserver.expect_request("/v1/system/info").respond_with_json(mock_system_info)

    computer = Computer(httpserver.url_for("/"))

    with pytest.raises(ValueError) as exc:
        computer.move_mouse(-1, 100)
    assert "X coordinate" in str(exc.value)

    with pytest.raises(ValueError) as exc:
        computer.move_mouse(100, 2000)
    assert "Y coordinate" in str(exc.value)
