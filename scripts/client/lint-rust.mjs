#!/usr/bin/env zx
import 'zx/globals';
import { getClippyToolchain, workingDirectory } from '../utils.mjs';

const channel = getClippyToolchain();
const toolchain = channel ? `+${channel}` : '';

// Check the client using Clippy.
cd(path.join(workingDirectory, 'clients', 'rust'));
await $`cargo ${toolchain} clippy ${process.argv.slice(3)}`;
