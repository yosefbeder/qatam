const { spawn } = require('child_process');
const { rm, writeFile } = require('fs/promises');

function isValidate(body) {
  const keys = Object.keys(body);
  if (keys.length == 1 && keys[0] == 'code') {
    return true;
  }
  return false;
}

async function executeCode(code) {
  const PATH = `${__dirname}\\تجربة.قتام`;
  await writeFile(PATH, code);
  const process = spawn('قتام', [PATH]);
  const output = { stdout: '', stderr: '' };

  setTimeout(async () => {
    if (!process.killed) {
      process.kill();
    }
  }, 1000);

  for await (const chunk of process.stdout) {
    output.stdout += chunk.toString();
  }

  for await (const chunk of process.stderr) {
    output.stderr += chunk.toString();
  }

  await rm(PATH);
  return { exitCode: process.exitCode, ...output };
}

async function execute(req, res) {
  const { body } = req;

  if (!isValidate(body)) {
    res.status(400).send('Invalid request');
    return;
  }

  const { code } = body;

  try {
    const output = await executeCode(code);
    res.json(output);
  } catch (err) {
    res.status(500).send(err.toString());
  }
}

exports.execute = execute;
