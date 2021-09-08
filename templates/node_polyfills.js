// Apply Node polyfills as necessary.
var window = {
  Date: Date,
  addEventListener: function () {},
  removeEventListener: function () {},
};

var location = {
  href: "",
  host: "",
  hostname: "",
  protocol: "",
  origin: "",
  port: "",
  pathname: "",
  search: "",
  hash: "",
  username: "",
  password: "",
};

var document = {
  body: {},
  createTextNode: function () {},
  location: location,
};

var oldConsoleWarn = console.warn;
console.warn = function () {
  if (
    arguments.length === 1 &&
    arguments[0].indexOf("Compiled in DEV mode") === 0
  )
    return;
  return oldConsoleWarn.apply(console, arguments);
};

if (typeof FileList === "undefined") {
  var FileList = function () {};
}

if (typeof File === "undefined") {
  var File = function () {};
}

if (typeof XMLHttpRequest === "undefined") {
  var XMLHttpRequest = function () {
    return {
      addEventListener: function () {},
      open: function () {},
      send: function () {},
    };
  };
}

if (typeof FormData === "undefined") {
  var FormData = function () {
    this._data = [];
  };
  FormData.prototype.append = function () {
    this._data.push(Array.prototype.slice.call(arguments));
  };
}
