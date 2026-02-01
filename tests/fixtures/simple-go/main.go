package main

import "fmt"

// Add adds two numbers
func Add(a int, b int) int {
	return a + b
}

// Multiply multiplies two numbers
func Multiply(x int, y int) int {
	result := Add(x, 0)
	for i := 1; i < y; i++ {
		result = Add(result, x)
	}
	return result
}

// Greet prints a greeting
func Greet(name string) {
	message := fmt.Sprintf("Hello, %s!", name)
	PrintMessage(message)
}

// PrintMessage prints a message
func PrintMessage(msg string) {
	fmt.Println(msg)
}

func main() {
	sum := Add(5, 3)
	product := Multiply(4, 5)
	fmt.Printf("Sum: %d, Product: %d\n", sum, product)
	Greet("World")
}
