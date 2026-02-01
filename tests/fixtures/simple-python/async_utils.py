"""Async utilities with decorators."""

import asyncio
from typing import Any, Dict


async def fetch_user(user_id: int) -> Dict[str, Any]:
    """Fetch user data asynchronously."""
    data = await fetch_data(f"/users/{user_id}")
    return process_user_data(data)


async def fetch_data(url: str) -> Dict[str, Any]:
    """Fetch data from API."""
    # Simulate async API call
    await asyncio.sleep(0.1)
    return {"data": "mock"}


def process_user_data(data: Dict[str, Any]) -> Dict[str, Any]:
    """Process user data."""
    return {**data, "processed": True}


async def validate_user(user: Dict[str, Any]) -> bool:
    """Validate user asynchronously."""
    is_valid = await check_validation(user)
    return is_valid


async def check_validation(user: Dict[str, Any]) -> bool:
    """Check if user is valid."""
    return user is not None


def decorator_example(func):
    """Example decorator."""
    def wrapper(*args, **kwargs):
        print(f"Calling {func.__name__}")
        result = func(*args, **kwargs)
        print(f"Result: {result}")
        return result
    return wrapper


@decorator_example
def greet(name: str) -> str:
    """Greet a person."""
    return format_greeting(name)


def format_greeting(name: str) -> str:
    """Format greeting message."""
    return f"Hello, {name}!"


async def main():
    """Main async function."""
    user = await fetch_user(1)
    is_valid = await validate_user(user)

    if is_valid:
        print("User is valid:", user)

    greeting = greet("World")
    print(greeting)


if __name__ == "__main__":
    asyncio.run(main())
