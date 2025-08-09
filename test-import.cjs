const fs=require("node:fs");
console.log(fs.readdirSync("."));
const {SystemTray}=require("./index");