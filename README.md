# JavaScript: Infinitum
JavaScript: Infinitum (JSI for short) is an alternative JavaScript language that compiles to JS/TS. It is designed to fix closure scope issues, fix `typeof null`, and more.

## Features
- [x] Fix closure scope issues
- [x] Fix `typeof null`
- [x] Add `hoist` keyword (see [Hoisting](#hoisting))

## Hoisting
The `hoist` keyword is used to hoist a variable or function to the top of the file.

The following JSI code:
```js
console.log(a); // Hello, World!
hoist let a = "Hello, World!";
```

Compiles to:
```js
let a = "Hello, World!";
console.log(a);
```
