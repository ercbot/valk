import os

import anthropic

from valk import Computer
from valk.integrations.anthropic import ComputerTool

# Get API key from env or prompt user
api_key = os.getenv("ANTHROPIC_API_KEY")
if not api_key:
    api_key = input("Please enter your Anthropic API key: ").strip()

# Initialize client
client = anthropic.Anthropic(api_key=api_key)

# System prompt provides context about the environment
SYSTEM_PROMPT = """You are utilizing an Ubuntu virtual machine using debian:bookworm-slim.
You have access to computer control capabilities through the computer tool.
"""


def handle_response(response, messages, computer_tool):
    """Recursively handle responses and tool calls"""
    messages.append({"role": "assistant", "content": response.content})

    for content_block in response.content:
        if content_block.type == "text":
            print("Claude:", content_block.text)
        elif content_block.type == "tool_use":
            print("- Tool: ", content_block.input)
            # Execute the tool call
            result = computer_tool(**content_block.input)

            # Prepare tool result content
            content = [
                {
                    "type": "tool_result",
                    "tool_use_id": content_block.id,
                    "content": result.output,
                }
            ]

            if result.base64_image:
                content.append(
                    {
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": "image/png",
                            "data": result.base64_image,
                        },
                    }
                )

            # Add tool result to messages
            messages.append({"role": "user", "content": content})

            # Get follow-up response and handle recursively
            follow_up = client.beta.messages.create(
                model="claude-3-5-sonnet-latest",
                messages=messages,
                system=SYSTEM_PROMPT,
                max_tokens=1024,
                tools=[computer_tool.to_params()],
                tool_choice={"disable_parallel_tool_use": True, "type": "auto"},
                betas=["computer-use-2024-10-22"],
            )

            handle_response(follow_up, messages, computer_tool)


def main():
    # Connect to the computer and start the debug viewer
    computer = Computer("http://localhost:8255")
    computer.start_debug_viewer()

    # Initialize tool
    computer_tool = ComputerTool(computer)
    messages = []

    print("Chat with Claude (type 'quit' to exit)")
    print("-" * 40)

    while True:
        user_input = input("You: ").strip()

        if user_input.lower() == "quit":
            break

        messages.append({"role": "user", "content": user_input})

        # Call Claude with computer use beta flag
        response = client.beta.messages.create(
            model="claude-3-5-sonnet-latest",
            messages=messages,
            system=SYSTEM_PROMPT,
            max_tokens=1024,
            tools=[computer_tool.to_params()],
            tool_choice={"disable_parallel_tool_use": True, "type": "auto"},
            betas=["computer-use-2024-10-22"],
        )

        handle_response(response, messages, computer_tool)

    print("Goodbye!")


if __name__ == "__main__":
    main()
