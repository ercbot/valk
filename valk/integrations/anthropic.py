from dataclasses import dataclass, fields, replace
from enum import Enum
from typing import Literal, NamedTuple, TypedDict

from valk.computer import Computer


class Resolution(NamedTuple):
    width: int
    height: int


class ScalingSource(str, Enum):
    COMPUTER = "computer"  # Scaling coordinates from computer to API
    API = "api"  # Scaling coordinates from API to computer


# Target resolutions for scaling down high-res displays
# These are common resolutions that provide good balance of visibility and performance
MAX_SCALING_TARGETS = {
    "XGA": Resolution(1024, 768),  # 4:3 aspect ratio
    "WXGA": Resolution(1280, 800),  # 16:10 aspect ratio
    "FWXGA": Resolution(1366, 768),  # ~16:9 aspect ratio
}


class ComputerToolOptions(TypedDict):
    display_height_px: int
    display_width_px: int
    display_number: int | None


@dataclass(kw_only=True, frozen=True)
class ToolResult:
    """Represents the result of a tool execution."""

    output: str | None = None
    error: str | None = None
    base64_image: str | None = None
    system: str | None = None

    def __bool__(self):
        return any(getattr(self, field.name) for field in fields(self))

    def __add__(self, other: "ToolResult"):
        def combine_fields(
            field: str | None, other_field: str | None, concatenate: bool = True
        ):
            if field and other_field:
                if concatenate:
                    return field + other_field
                raise ValueError("Cannot combine tool results")
            return field or other_field

        return ToolResult(
            output=combine_fields(self.output, other.output),
            error=combine_fields(self.error, other.error),
            base64_image=combine_fields(self.base64_image, other.base64_image, False),
            system=combine_fields(self.system, other.system),
        )

    def replace(self, **kwargs):
        """Returns a new ToolResult with the given fields replaced."""
        return replace(self, **kwargs)


class ToolFailure(ToolResult):
    """A ToolResult that represents a failure."""

    pass


class ToolError(Exception):
    """Raised when a tool encounters an error."""

    def __init__(self, message):
        self.message = message


# Simple Computer Tool implementation
class ComputerTool:
    name: Literal["computer"] = "computer"
    api_type: Literal["computer_20241022"] = "computer_20241022"
    width: int
    height: int
    display_num: int | None
    _scaling_enabled: bool = True  # Enable/disable coordinate scaling

    def __init__(self, computer: Computer):
        self.computer = computer
        self.width = computer.system_info.display_width
        self.height = computer.system_info.display_height
        self.display_num = None

    def __call__(self, **kwargs) -> ToolResult:
        action = kwargs.get("action")
        text = kwargs.get("text")
        coordinate = kwargs.get("coordinate")

        try:
            match action:
                case "mouse_move" if coordinate:
                    # Scale coordinates from API to computer resolution
                    x, y = self.scale_coordinates(
                        ScalingSource.API, coordinate[0], coordinate[1]
                    )
                    self.computer.move_mouse(x, y)
                    return ToolResult(output=f"Moved mouse to {x}, {y}")

                case "left_click":
                    self.computer.left_click()
                    return ToolResult(output="Performed left click")

                case "right_click":
                    self.computer.right_click()
                    return ToolResult(output="Performed right click")

                case "middle_click":
                    self.computer.middle_click()
                    return ToolResult(output="Performed middle click")

                case "double_click":
                    self.computer.double_click()
                    return ToolResult(output="Performed double click")

                case "left_click_drag" if coordinate:
                    # Scale drag coordinates from API to computer resolution
                    x, y = self.scale_coordinates(
                        ScalingSource.API, coordinate[0], coordinate[1]
                    )
                    self.computer.left_click_drag(x, y)
                    return ToolResult(output=f"Dragged to {x}, {y}")

                case "type" if text:
                    self.computer.type(text)
                    return ToolResult(output=f"Typed text: {text}")

                case "key" if text:
                    self.computer.key(text)
                    return ToolResult(output=f"Pressed key: {text}")

                case "screenshot":
                    base64_image = self.computer.screenshot()

                    return ToolResult(
                        output="Screenshot taken", base64_image=base64_image
                    )

                case "cursor_position":
                    # Scale cursor position from computer to API resolution
                    x, y = self.computer.get_cursor_position()
                    scaled_x, scaled_y = self.scale_coordinates(
                        ScalingSource.COMPUTER, x, y
                    )
                    return ToolResult(output=f"Cursor position: {scaled_x}, {scaled_y}")

                case _:
                    return ToolFailure(
                        error=f"Invalid action or missing required parameters: {action}"
                    )

        except Exception as e:
            return ToolFailure(error=str(e))

    def scale_coordinates(
        self, source: ScalingSource, x: int, y: int
    ) -> tuple[int, int]:
        """
        Scale coordinates between API and computer resolutions to handle high-DPI displays.

        Args:
            source: Whether coordinates are coming from computer or API
            x: X coordinate to scale
            y: Y coordinate to scale

        Returns:
            Tuple of (scaled_x, scaled_y)
        """
        if not self._scaling_enabled:
            return x, y

        # Calculate aspect ratio of current display
        ratio = self.width / self.height
        target_resolution = None

        # Find appropriate target resolution matching aspect ratio
        for resolution in MAX_SCALING_TARGETS.values():
            # Allow small deviation in aspect ratio (not all are exactly 16:9 etc)
            if abs(resolution.width / resolution.height - ratio) < 0.02:
                if resolution.width < self.width:
                    target_resolution = resolution
                break

        if target_resolution is None:
            return x, y  # No scaling needed

        # Calculate scaling factors (will be < 1 for downscaling)
        x_scale = target_resolution.width / self.width
        y_scale = target_resolution.height / self.height

        if source == ScalingSource.API:
            # Scale up from API coordinates to computer coordinates
            if x > self.width or y > self.height:
                raise ValueError(f"Coordinates {x}, {y} are out of bounds")
            return round(x / x_scale), round(y / y_scale)
        else:
            # Scale down from computer coordinates to API coordinates
            return round(x * x_scale), round(y * y_scale)

    @property
    def options(self) -> ComputerToolOptions:
        # Scale display dimensions for API consumption
        width, height = self.scale_coordinates(
            ScalingSource.COMPUTER, self.width, self.height
        )
        return {
            "display_width_px": width,
            "display_height_px": height,
            "display_number": self.display_num,
        }

    def to_params(self) -> dict:
        return {"name": self.name, "type": self.api_type, **self.options}
