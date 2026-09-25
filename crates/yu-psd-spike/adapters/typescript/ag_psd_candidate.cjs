"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");

const UNAVAILABLE_EXIT = 3;

function unavailable(message) {
  console.error(message);
  process.exit(UNAVAILABLE_EXIT);
}

function readArg(name, required = true) {
  const index = process.argv.indexOf(name);
  if (index === -1 || index + 1 >= process.argv.length) {
    if (required) unavailable(`missing required argument: ${name}`);
    return undefined;
  }
  return process.argv[index + 1];
}

const expectedVersion = readArg("--expected-version");
const expectedNodeMajor = readArg("--expected-node-major");
const mode = readArg("--mode", false) || "inspect";
const layerName = readArg("--layer-name", false);
const warmups = Number(readArg("--warmups", false) || "0");
const samples = Number(readArg("--samples", false) || "0");
const input = process.argv[process.argv.length - 1];

const actualNodeMajor = process.versions.node.split(".")[0];
if (actualNodeMajor !== expectedNodeMajor) {
  unavailable(
    `Node.js version mismatch: expected major ${expectedNodeMajor}, got ${process.versions.node}`
  );
}

let readPsd;
let getLayerImageData;
let packageVersion;
try {
  ({ readPsd, getLayerImageData } = require("ag-psd"));
  packageVersion = require("ag-psd/package.json").version;
} catch (error) {
  unavailable(
    `ag-psd is unavailable: ${error instanceof Error ? error.message : String(error)}`
  );
}

if (packageVersion !== expectedVersion) {
  unavailable(
    `ag-psd version mismatch: expected ${expectedVersion}, got ${packageVersion}`
  );
}

function hasBitmapMask(layer) {
  const mask = layer && layer.mask;
  const realMask = layer && layer.realMask;

  return Boolean(
    (mask && mask.fromVectorData !== true) ||
    (realMask && realMask.fromVectorData !== true)
  );
}

function flattenLayers(psd) {
  const layers = [];
  let maximumTreeDepth = 0;

  function visit(children, depth) {
    if (!Array.isArray(children)) return;

    for (const layer of children) {
      layers.push({ layer, depth });
      maximumTreeDepth = Math.max(maximumTreeDepth, depth);
      visit(layer.children, depth + 1);
    }
  }

  visit(psd.children, 1);
  return { layers, maximumTreeDepth };
}

function inspectPsd(psd) {
  const { layers, maximumTreeDepth } = flattenLayers(psd);
  return {
    parse_success: true,
    width: psd.width,
    height: psd.height,
    layer_count: layers.length,
    maximum_tree_depth: maximumTreeDepth,
    layer_names: layers.map(({ layer }) =>
      typeof layer.name === "string" ? layer.name : ""
    ),
    text_layer_count: layers.filter(({ layer }) => Boolean(layer.text)).length,
    pixel_mask_layer_count: layers.filter(({ layer }) => hasBitmapMask(layer)).length,
    vector_mask_layer_count: layers.filter(({ layer }) => Boolean(layer.vectorMask)).length,
  };
}

function findLayer(psd, name) {
  const { layers } = flattenLayers(psd);
  const matches = layers
    .map(({ layer }) => layer)
    .filter((layer) => layer.name === name);

  if (matches.length !== 1) {
    throw new Error(
      `layer selector ${JSON.stringify(name)} matched ${matches.length} layers; expected exactly one`
    );
  }
  return matches[0];
}

function readStructure(buffer) {
  return readPsd(buffer, {
    skipLayerImageData: true,
    skipCompositeImageData: true,
    skipThumbnail: true,
    skipLinkedFilesData: true,
    logMissingFeatures: false,
  });
}

function readForExport(buffer) {
  return readPsd(buffer, {
    useRawData: true,
    skipCompositeImageData: true,
    skipThumbnail: true,
    skipLinkedFilesData: true,
    logMissingFeatures: false,
  });
}

function exportLayer(buffer, name) {
  const psd = readForExport(buffer);
  const layer = findLayer(psd, name);
  const image = getLayerImageData(layer);
  if (!image) {
    throw new Error(`layer ${JSON.stringify(name)} has no exportable pixel data`);
  }
  if (image.data.BYTES_PER_ELEMENT !== 1) {
    throw new Error(
      `layer ${JSON.stringify(name)} is not rgba8 (BYTES_PER_ELEMENT=${image.data.BYTES_PER_ELEMENT})`
    );
  }

  const raw = Buffer.from(
    image.data.buffer,
    image.data.byteOffset,
    image.data.byteLength
  );
  return {
    layer_name: name,
    width: image.width,
    height: image.height,
    pixel_format: "rgba8",
    rgba_sha256: crypto.createHash("sha256").update(raw).digest("hex"),
    rgba_bytes: raw.length,
  };
}

function nowNs() {
  return process.hrtime.bigint();
}

function elapsedUs(started) {
  return Number((process.hrtime.bigint() - started) / 1000n);
}

function benchmark(buffer, name, warmupIterations, sampleIterations) {
  for (let i = 0; i < warmupIterations; i += 1) {
    inspectPsd(readStructure(buffer));
  }

  const inspectSamplesUs = [];
  for (let i = 0; i < sampleIterations; i += 1) {
    const started = nowNs();
    inspectPsd(readStructure(buffer));
    inspectSamplesUs.push(elapsedUs(started));
  }

  for (let i = 0; i < warmupIterations; i += 1) {
    exportLayer(buffer, name);
  }

  const layerExportSamplesUs = [];
  let layerExport = null;
  for (let i = 0; i < sampleIterations; i += 1) {
    const started = nowNs();
    layerExport = exportLayer(buffer, name);
    layerExportSamplesUs.push(elapsedUs(started));
  }

  return {
    measurement: "warm_runtime_operation",
    input_read: "once_before_timing",
    runtime_startup: "excluded",
    warmup_iterations: warmupIterations,
    sample_iterations: sampleIterations,
    inspect_samples_us: inspectSamplesUs,
    layer_export_samples_us: layerExportSamplesUs,
    layer_export: layerExport,
  };
}

let buffer;
try {
  buffer = fs.readFileSync(input);
} catch (error) {
  console.error(`ag-psd failed to read input: ${error.message}`);
  process.exit(4);
}

if (mode === "benchmark") {
  if (!layerName) unavailable("--layer-name is required for benchmark mode");
  if (!Number.isInteger(warmups) || warmups < 0 || !Number.isInteger(samples) || samples <= 0) {
    unavailable("benchmark requires integer --warmups >= 0 and --samples > 0");
  }

  try {
    process.stdout.write(JSON.stringify(benchmark(buffer, layerName, warmups, samples)));
    process.exit(0);
  } catch (error) {
    console.error(
      `ag-psd benchmark failed: ${error instanceof Error ? error.message : String(error)}`
    );
    process.exit(4);
  }
}

if (mode === "export-layer") {
  if (!layerName) unavailable("--layer-name is required for export-layer mode");
  try {
    process.stdout.write(JSON.stringify(exportLayer(buffer, layerName)));
    process.exit(0);
  } catch (error) {
    console.error(
      `ag-psd layer export failed: ${error instanceof Error ? error.message : String(error)}`
    );
    process.exit(4);
  }
}

try {
  process.stdout.write(JSON.stringify(inspectPsd(readStructure(buffer))));
} catch (_error) {
  process.stdout.write(JSON.stringify({ parse_success: false }));
}
