---
title: "JavaScript Modern Features"
description: "ES6+ features guide covering arrow functions, destructuring, async/await, and more"
author: "Typstify Team"
tags: ["javascript", "es6", "modern", "web development"]
draft: false
---

## ES6+ Features Guide

JavaScript has evolved significantly with ES6 and beyond. Here are the key features you should know.

## Arrow Functions

Arrow functions provide a more concise syntax:

```javascript
// Traditional function
function add(a, b) {
    return a + b;
}

// Arrow function
const add = (a, b) => a + b;

// With block body
const multiply = (a, b) => {
    const result = a * b;
    return result;
};

// Array methods with arrow functions
const numbers = [1, 2, 3, 4, 5];
const doubled = numbers.map(n => n * 2);
const evens = numbers.filter(n => n % 2 === 0);
const sum = numbers.reduce((acc, n) => acc + n, 0);

console.log(doubled); // [2, 4, 6, 8, 10]
console.log(evens);   // [2, 4]
console.log(sum);     // 15
```

## Destructuring

Extract values from arrays and objects:

```javascript
// Array destructuring
const [first, second, ...rest] = [1, 2, 3, 4, 5];
console.log(first);  // 1
console.log(second); // 2
console.log(rest);   // [3, 4, 5]

// Object destructuring
const person = { name: 'Alice', age: 30, city: 'New York' };
const { name, age, city } = person;
console.log(name); // 'Alice'

// With renaming
const { name: personName, age: personAge } = person;
console.log(personName); // 'Alice'

// Function parameters
function greet({ name, age }) {
    return `Hello ${name}, you are ${age} years old`;
}

console.log(greet(person)); // 'Hello Alice, you are 30 years old'
```

## Template Literals

String interpolation and multiline strings:

```javascript
const name = 'World';
const greeting = `Hello, ${name}!`;

// Multiline strings
const html = `
    <div class="container">
        <h1>${greeting}</h1>
        <p>This is a multiline string</p>
    </div>
`;

// Tagged template literals
function highlight(strings, ...values) {
    return strings.reduce((result, string, i) => {
        return result + string + (values[i] ? `<mark>${values[i]}</mark>` : '');
    }, '');
}

const user = 'John';
const action = 'logged in';
const message = highlight`User ${user} has ${action}`;
// Result: "User <mark>John</mark> has <mark>logged in</mark>"
```

## Async/Await

Modern asynchronous programming:

```javascript
// Promise-based approach
function fetchUserData(id) {
    return fetch(`/api/users/${id}`)
        .then(response => response.json())
        .then(data => data)
        .catch(error => {
            console.error('Error:', error);
            throw error;
        });
}

// Async/await approach
async function fetchUserDataAsync(id) {
    try {
        const response = await fetch(`/api/users/${id}`);
        const data = await response.json();
        return data;
    } catch (error) {
        console.error('Error:', error);
        throw error;
    }
}

// Usage
async function displayUser(id) {
    try {
        const user = await fetchUserDataAsync(id);
        console.log('User:', user.name);
    } catch (error) {
        console.log('Failed to load user');
    }
}

// Multiple async operations
async function fetchMultipleUsers(ids) {
    const promises = ids.map(id => fetchUserDataAsync(id));
    const users = await Promise.all(promises);
    return users;
}
```

## Classes

Object-oriented programming in JavaScript:

```javascript
class Animal {
    constructor(name, species) {
        this.name = name;
        this.species = species;
    }
    
    speak() {
        console.log(`${this.name} makes a sound`);
    }
    
    // Getter
    get info() {
        return `${this.name} is a ${this.species}`;
    }
    
    // Static method
    static compare(animal1, animal2) {
        return animal1.species === animal2.species;
    }
}

class Dog extends Animal {
    constructor(name, breed) {
        super(name, 'dog');
        this.breed = breed;
    }
    
    speak() {
        console.log(`${this.name} barks`);
    }
    
    fetch() {
        console.log(`${this.name} fetches the ball`);
    }
}

const dog = new Dog('Rex', 'German Shepherd');
dog.speak(); // 'Rex barks'
dog.fetch(); // 'Rex fetches the ball'
console.log(dog.info); // 'Rex is a dog'
```

## Modules

Organize code with import/export:

```javascript
// math.js
export const PI = 3.14159;

export function add(a, b) {
    return a + b;
}

export function multiply(a, b) {
    return a * b;
}

export default function subtract(a, b) {
    return a - b;
}

// main.js
import subtract, { PI, add, multiply } from './math.js';

// Or import everything
import * as math from './math.js';

console.log(add(5, 3));        // 8
console.log(subtract(10, 4));  // 6
console.log(PI);               // 3.14159

// Using namespace import
console.log(math.multiply(4, 3)); // 12
console.log(math.default(8, 2));  // 6 (default export)
```

## Advanced Features

### Symbols

```javascript
// Create unique symbols
const sym1 = Symbol('description');
const sym2 = Symbol('description');
console.log(sym1 === sym2); // false

// Use as object keys
const obj = {
    [sym1]: 'value1',
    [sym2]: 'value2'
};

console.log(obj[sym1]); // 'value1'
```

### Iterators and Generators

```javascript
// Generator function
function* fibonacci() {
    let a = 0, b = 1;
    while (true) {
        yield a;
        [a, b] = [b, a + b];
    }
}

// Usage
const fib = fibonacci();
console.log(fib.next().value); // 0
console.log(fib.next().value); // 1
console.log(fib.next().value); // 1
console.log(fib.next().value); // 2

// Custom iterator
const range = {
    from: 1,
    to: 5,
    
    [Symbol.iterator]() {
        return {
            current: this.from,
            last: this.to,
            
            next() {
                if (this.current <= this.last) {
                    return { done: false, value: this.current++ };
                } else {
                    return { done: true };
                }
            }
        };
    }
};

for (let num of range) {
    console.log(num); // 1, 2, 3, 4, 5
}
```

### Proxy

```javascript
const target = {
    name: 'John',
    age: 30
};

const handler = {
    get(target, prop) {
        console.log(`Getting ${prop}`);
        return target[prop];
    },
    
    set(target, prop, value) {
        console.log(`Setting ${prop} to ${value}`);
        if (prop === 'age' && value < 0) {
            throw new Error('Age cannot be negative');
        }
        target[prop] = value;
        return true;
    }
};

const proxy = new Proxy(target, handler);
console.log(proxy.name); // "Getting name" then "John"
proxy.age = 25;          // "Setting age to 25"
```

## Best Practices

> **Performance Tip**: Use `const` and `let` instead of `var`. This provides better scoping and prevents hoisting issues.

### Code Organization

1. **Use modules** to organize related functionality
2. **Prefer composition over inheritance**
3. **Use meaningful variable and function names**
4. **Write tests** for your functions
5. **Handle errors gracefully**

### Modern Patterns

```javascript
// Optional chaining
const user = {
    profile: {
        social: {
            twitter: '@johndoe'
        }
    }
};

console.log(user?.profile?.social?.twitter); // '@johndoe'
console.log(user?.profile?.social?.facebook); // undefined

// Nullish coalescing
const defaultName = user?.name ?? 'Anonymous';
const defaultAge = user?.age ?? 0;

// Array methods chaining
const processedData = data
    .filter(item => item.active)
    .map(item => ({ ...item, processed: true }))
    .sort((a, b) => a.priority - b.priority);
```

## Resources

- [MDN JavaScript Guide](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide)
- [ES6 Features](http://es6-features.org/)
- [You Don't Know JS](https://github.com/getify/You-Dont-Know-JS)
- [JavaScript.info](https://javascript.info/)
