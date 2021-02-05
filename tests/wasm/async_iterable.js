exports.createAsyncIterable = async function* (iterable) {
  for (const elem of iterable) {
    yield elem;
  }
}
