// Calculator utilities

/**
 * Adds two numbers
 */
export function add(a: number, b: number): number {
  return a + b;
}

/**
 * Subtracts two numbers
 */
export function subtract(a: number, b: number): number {
  return a - b;
}

/**
 * Multiplies two numbers using repeated addition
 */
export function multiply(x: number, y: number): number {
  let result = 0;
  for (let i = 0; i < y; i++) {
    result = add(result, x);
  }
  return result;
}

/**
 * Divides two numbers
 */
export function divide(a: number, b: number): number {
  if (b === 0) {
    throw new Error("Division by zero");
  }
  return a / b;
}

/**
 * Calculates power using repeated multiplication
 */
export const power = (base: number, exponent: number): number => {
  if (exponent === 0) return 1;

  let result = base;
  for (let i = 1; i < exponent; i++) {
    result = multiply(result, base);
  }
  return result;
};

/**
 * Calculator class
 */
export class Calculator {
  private history: string[] = [];

  constructor() {}

  add(a: number, b: number): number {
    const result = add(a, b);
    this.logOperation(`${a} + ${b} = ${result}`);
    return result;
  }

  subtract(a: number, b: number): number {
    const result = subtract(a, b);
    this.logOperation(`${a} - ${b} = ${result}`);
    return result;
  }

  multiply(a: number, b: number): number {
    const result = multiply(a, b);
    this.logOperation(`${a} * ${b} = ${result}`);
    return result;
  }

  private logOperation(operation: string): void {
    this.history.push(operation);
    console.log(operation);
  }

  getHistory(): string[] {
    return this.history;
  }
}

// Usage example
const calc = new Calculator();
const sum = calc.add(5, 3);
const product = calc.multiply(4, 5);
console.log("Sum:", sum);
console.log("Product:", product);
