import { engine  } from './dist/seedwing_policy-engine-component.js';

console.log(`Seedwing Policy Engine version: ${engine.version()}`);
const policies = []; // The wit enabled engine does not support this yet.
const datas = [];    // The wit enabled engine does not support this yet.
const policy = 'pattern dog = { name: string, trained: boolean }';
const name = "dog"
const input = JSON.stringify({
  name: "goodboy",
  trained: true
});
const result = engine.eval(policies, datas, policy, name, input);
console.log(result);
