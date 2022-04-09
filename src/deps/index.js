"use strict";

function safeUseClass(clazz) {
  try {
    return Java.use(clazz);
  } catch (_e) {
    console.log(`error: "${clazz}" not found`);
    return null;
  }
}

function monitorLibcOpen() {
  // https://github.com/iddoeldor/frida-snippets#intercept-open
  const f = Module.findExportByName(
    "/apex/com.android.runtime/lib64/bionic/libc.so",
    "open"
  );
  if (!f) {
    console.log("libc::open not found");
    return;
  }

  console.log("[C] libc::open @", f);
  Interceptor.attach(f, {
    onEnter: function (args) {
      this.flag = false;
      const filename = Memory.readCString(ptr(args[0]));
      console.log("libc@open: ", filename);
    },
    onLeave: function (retval) {
      if (this.flag)
        // passed from onEnter
        console.warn("\nretval: " + retval);
    },
  });
}

function monitorJavaMethod(method) {
  console.log("[J] " + method);
  const pos = method.lastIndexOf(".");
  const clazz = method.slice(0, pos);
  const methodName = method.slice(pos + 1);

  const vm = safeUseClass(clazz);
  if (!vm) {
    return;
  }
  vm[methodName].overloads.forEach((m) => {
    m.implementation = function () {
      let args = [];
      for (let i = 0; i < arguments.length; ++i) {
        args.push(`(${i})` + arguments[i]);
      }

      console.log(`enter [${method}]: ` + args.join(", "));
      let ret = this[methodName].apply(this, arguments);
      console.log(`return [${method}]: ` + ret);
      return ret;
    };
  });
}

function monitorJavaClass(clazz) {
  let vm = safeUseClass(clazz);
  if (!vm) {
    return;
  }
  vm.class.getDeclaredMethods().forEach((m) => {
    monitorJavaMethod(clazz + "." + m.getName());
  });
}

function main() {
  console.log("!! frida script is loaded.");
  Java.perform(() => {
    console.log("vm is attached");
    monitorJavaClass("com.a.b");
    monitorLibcOpen();
  });
}

main();
