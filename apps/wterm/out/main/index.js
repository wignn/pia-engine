"use strict";
const electron = require("electron");
const path = require("path");
const utils = require("@electron-toolkit/utils");
const icon = path.join(__dirname, "../../resources/icon.png");
function windowUrl() {
  return utils.is.dev && process.env["ELECTRON_RENDERER_URL"] ? process.env["ELECTRON_RENDERER_URL"] : `file://${path.join(__dirname, "../renderer/index.html")}`;
}
function createPopout(payload) {
  const panel = payload.panel || "panel";
  const win = new electron.BrowserWindow({
    width: panel === "chart" ? 980 : 760,
    height: panel === "chart" ? 680 : 560,
    minWidth: 520,
    minHeight: 380,
    title: `WTERM ${payload.symbol || ""} ${payload.fn || panel}`.trim(),
    autoHideMenuBar: true,
    ...process.platform === "linux" ? { icon } : {},
    webPreferences: {
      preload: path.join(__dirname, "../preload/index.js"),
      sandbox: false
    }
  });
  const url = new URL(windowUrl());
  url.hash = `popout=${encodeURIComponent(panel)}&symbol=${encodeURIComponent(payload.symbol || "")}&fn=${encodeURIComponent(payload.fn || "")}`;
  if (utils.is.dev && process.env["ELECTRON_RENDERER_URL"]) {
    win.loadURL(url.toString());
  } else {
    win.loadFile(path.join(__dirname, "../renderer/index.html"), { hash: url.hash });
  }
}
function createWindow() {
  const mainWindow = new electron.BrowserWindow({
    width: 1440,
    height: 920,
    minWidth: 1180,
    minHeight: 720,
    show: false,
    autoHideMenuBar: true,
    ...process.platform === "linux" ? { icon } : {},
    webPreferences: {
      preload: path.join(__dirname, "../preload/index.js"),
      sandbox: false
    }
  });
  mainWindow.on("ready-to-show", () => {
    mainWindow.show();
  });
  mainWindow.webContents.setWindowOpenHandler((details) => {
    electron.shell.openExternal(details.url);
    return { action: "deny" };
  });
  if (utils.is.dev && process.env["ELECTRON_RENDERER_URL"]) {
    mainWindow.loadURL(process.env["ELECTRON_RENDERER_URL"]);
  } else {
    mainWindow.loadFile(path.join(__dirname, "../renderer/index.html"));
  }
}
electron.app.whenReady().then(() => {
  utils.electronApp.setAppUserModelId("com.atlsd.wterm");
  electron.app.on("browser-window-created", (_, window) => {
    utils.optimizer.watchWindowShortcuts(window);
  });
  electron.ipcMain.on("wterm:popout", (_, payload) => createPopout(payload || {}));
  createWindow();
  electron.app.on("activate", function() {
    if (electron.BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});
electron.app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    electron.app.quit();
  }
});
