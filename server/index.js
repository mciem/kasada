const express = require("express");
const path = require("path");

const app = express();

app.use(express.static(path.join(__dirname, "assets")));

app.get("/", (req, res) => {
  res.sendFile(path.join(__dirname, "assets", "index.html"));
});

app.listen(3001, () => console.log("Server started at http://localhost:3000"));
