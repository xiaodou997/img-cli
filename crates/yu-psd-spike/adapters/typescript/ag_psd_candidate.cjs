"use strict";

const fs = require("node:fs");

const UNAVAILABLE_EXIT = 3;

function unavailable(message) {
  console.error(message);
  process.exit(UNAVAILABLE_EXIT);
}

function readArg(name) {
  const index = process.argv.indexOf(name);
  if (index === -1 || index + 1 >= process.argv.length) {
    unavailable(`missing required argument: ${name}`);
  }
  return process.argv[index + 1];
}

const expectedVersion = readArg("--expected-version");
const expectedNodeMajor = readArg("--expected-node-major");
const input = process.argv[process.argv.length - 1];

const actualNodeMajor = process.versions.node.split(".")[0];
if (actualNodeMajor !== expectedNodeMajor) {
  unavailable(
    `Node.js version mismatch: expected major ${expectedNodeMajor}, got ${process.versions.node}`
  );
}

let readPsd;
let packageVersion;
try {
  ({ readPsd } = require("ag-psd"));
  packageVersion = require("ag-psd/package.json").version;
} catch (error) {
  unavailable(`ag-psd is unavailable: ${error instanceof Error ? error.message : String(error)}`);
}

if (packageVersion !== expectedVersion) {
  unavailable(
    `ag-psd version mismatch: expected ${expectedVersion}, got ${packageVersion}`
  );
}

let psd;
try {
  const buffer = fs.readFileSync(input);
  psd = readPsd(buffer, {
    skipLayerImageData: true,
    skipCompositeImageData: true,
    skipThumbnail: true,
    skipLinkedFilesData: true,
    logMissingFeatures: false,
  });
} catch (_error) {
  process.stdout.write(JSON.stringify({ parse_success: false }));
  process.exit(0);
}

let layerCount = 0;
let maximumTreeDepth = 0;
let textLayerCount = 0;
let pixelMaskLayerCount = 0;
let vectorMaskLayerCount = 0;
const layerNames = [];

function hasBitmapMask(layer) {
  const mask = layer && layer.mask;
  const realMask = layer && layer.realMask;

  return Boolean(
    (mask && mask.fromVectorData !== true) ||
    (realMask && realMask.fromVectorData !== true)
  );
}

function visit(children, depth) {
  if (!Array.isArray(children)) return;

  for (const layer of children) {
    layerCount += 1;
    maximumTreeDepth = Math.max(maximumTreeDepth, depth);
    layerNames.push(typeof layer.name === "string" ? layer.name : "");

    if (layer.text) textLayerCount += 1;
    if (hasBitmapMask(layer)) pixelMaskLayerCount += 1;
    if (layer.vectorMask) vectorMaskLayerCount += 1;

    visit(layer.children, depth + 1);
  }
}

visit(psd.children, 1);

process.stdout.write(
  JSON.stringify({
    parse_success: true,
    width: psd.width,
    height: psd.height,
    layer_count: layerCount,
    maximum_tree_depth: maximumTreeDepth,
    layer_names: layerNames,
    text_layer_count: textLayerCount,
    pixel_mask_layer_count: pixelMaskLayerCount,
    vector_mask_layer_count: vectorMaskLayerCount,
  })
);
