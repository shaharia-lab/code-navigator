package main

import "fmt"

// Calculator is a simple calculator struct
type Calculator struct {
	name string
}

// NewCalculator creates a new Calculator
func NewCalculator(name string) *Calculator {
	return &Calculator{name: name}
}

// Add adds two numbers (method)
func (c *Calculator) Add(a, b int) int {
	result := a + b
	c.LogOperation("Add", result)
	return result
}

// Subtract subtracts two numbers (method)
func (c *Calculator) Subtract(a, b int) int {
	result := a - b
	c.LogOperation("Subtract", result)
	return result
}

// LogOperation logs an operation
func (c *Calculator) LogOperation(op string, result int) {
	msg := fmt.Sprintf("%s: %s = %d", c.name, op, result)
	PrintMessage(msg)
}
