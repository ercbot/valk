# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "anthropic",
# ]
# ///
import os
import json
import anthropic
from typing import Dict, Any, Literal
from cuse.integrations.anthropic import ComputerTool


# Get API key from env or prompt user
api_key = os.getenv("ANTHROPIC_API_KEY")
if not api_key:
    api_key = input("Please enter your Anthropic API key: ").strip()

# Initialize client
client = anthropic.Anthropic(api_key=api_key)
messages = []

# System prompt provides context about the environment
SYSTEM_PROMPT = """You are utilizing an Ubuntu virtual machine using debian:bookworm-slim.
You have access to computer control capabilities through the computer tool.
"""

# Initialize tools
computer = ComputerTool()

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
        tools=[computer.to_params()],
        betas=["computer-use-2024-10-22"],
    )

    assistant_message = response.content[0].text

    # Check for tool use in the response
    if hasattr(response.content[0], "tool_calls") and response.content[0].tool_calls:
        for tool_call in response.content[0].tool_calls:
            # Execute the tool call
            result = computer.run(**json.loads(tool_call.parameters))

            # Add tool result to messages
            messages.append(
                {
                    "role": "tool",
                    "content": result["output"]
                    if not result["error"]
                    else result["error"],
                    "tool_name": "computer",
                }
            )

            # Get follow-up response from Claude
            response = client.messages.create(
                model="claude-3-5-sonnet-latest",
                messages=messages,
                system=SYSTEM_PROMPT,
                max_tokens=1024,
                tools=[
                    {
                        "type": "function",
                        "function": {
                            "name": "computer",
                            "description": "Control computer actions",
                            "parameters": {
                                "type": "object",
                                "properties": {
                                    "action": {
                                        "type": "string",
                                        "description": "The action to perform",
                                    }
                                },
                            },
                        },
                    }
                ],
                betas=["computer-use-2024-10-22"],
            )
            assistant_message = response.content[0].text

    messages.append({"role": "assistant", "content": assistant_message})
    print("Claude:", assistant_message)

print("Goodbye!")
