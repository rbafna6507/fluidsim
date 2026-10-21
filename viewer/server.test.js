const test = require('node:test');
const assert = require('node:assert/strict');
const { parseOrientation } = require('./server');

test('accepts finite pitch and roll values from an orientation datagram', () => {
  assert.deepEqual(parseOrientation(Buffer.from('{"pitch":0.25,"roll":-0.5}')), {
    pitch: 0.25,
    roll: -0.5,
  });
});

test('rejects malformed or non-finite orientation datagrams', () => {
  assert.equal(parseOrientation(Buffer.from('not json')), null);
  assert.equal(parseOrientation(Buffer.from('{"pitch":null,"roll":0}')), null);
  assert.equal(parseOrientation(Buffer.from('{"pitch":0,"roll":"left"}')), null);
});
