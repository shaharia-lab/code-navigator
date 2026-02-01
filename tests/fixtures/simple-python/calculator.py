"""Calculator module with various Python patterns."""

def add(a: int, b: int) -> int:
    """Add two numbers."""
    return a + b


def subtract(a: int, b: int) -> int:
    """Subtract two numbers."""
    return a - b


def multiply(x: int, y: int) -> int:
    """Multiply using repeated addition."""
    result = 0
    for i in range(y):
        result = add(result, x)
    return result


def divide(a: float, b: float) -> float:
    """Divide two numbers."""
    if b == 0:
        raise ValueError("Division by zero")
    return a / b


def power(base: int, exponent: int) -> int:
    """Calculate power using repeated multiplication."""
    if exponent == 0:
        return 1

    result = base
    for i in range(1, exponent):
        result = multiply(result, base)
    return result


class Calculator:
    """Calculator class with instance methods."""

    def __init__(self):
        self.history = []

    def add(self, a: int, b: int) -> int:
        """Add two numbers and log."""
        result = add(a, b)
        self.log_operation(f"{a} + {b} = {result}")
        return result

    def subtract(self, a: int, b: int) -> int:
        """Subtract two numbers and log."""
        result = subtract(a, b)
        self.log_operation(f"{a} - {b} = {result}")
        return result

    def multiply(self, a: int, b: int) -> int:
        """Multiply two numbers and log."""
        result = multiply(a, b)
        self.log_operation(f"{a} * {b} = {result}")
        return result

    def log_operation(self, operation: str) -> None:
        """Log an operation."""
        self.history.append(operation)
        print(operation)

    def get_history(self) -> list:
        """Get operation history."""
        return self.history


# Usage
if __name__ == "__main__":
    calc = Calculator()
    sum_result = calc.add(5, 3)
    product = calc.multiply(4, 5)
    print(f"Sum: {sum_result}, Product: {product}")
