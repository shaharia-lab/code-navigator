// JavaScript utilities

function add(a, b) {
  return a + b;
}

function multiply(x, y) {
  let result = 0;
  for (let i = 0; i < y; i++) {
    result = add(result, x);
  }
  return result;
}

const greet = (name) => {
  const message = formatMessage(name);
  console.log(message);
};

function formatMessage(name) {
  return `Hello, ${name}!`;
}

class Calculator {
  constructor() {
    this.history = [];
  }

  add(a, b) {
    const result = add(a, b);
    this.logOperation(`${a} + ${b} = ${result}`);
    return result;
  }

  multiply(a, b) {
    const result = multiply(a, b);
    this.logOperation(`${a} * ${b} = ${result}`);
    return result;
  }

  logOperation(op) {
    this.history.push(op);
  }
}

module.exports = {
  add,
  multiply,
  greet,
  Calculator,
};
