const { Binary } = require('binary-install');
const os = require('os');

function getPlatform() {
	const type = os.type();
	const arch = os.arch();

	if (type === 'Windows_NT') {
		if (arch === 'x64') {
			return 'win64';
		} else {
			return 'win32';
		}
	}

	if (type === 'Linux' && arch === 'x64') {
		return 'linux';
	}

	if (type === 'Darwin' && arch === 'x64') {
		return 'macosx86';
	}

	if (type === 'Darwin' && arch === 'arm64') {
		return 'macosM1';
	}

	throw new Error(`Unsupported platform: ${type} ${arch}. Please create an issue at https://github.com/woubuc/sweep/issues`);
}

function getBinary() {
	const platform = getPlatform();
	const version = require('../package.json').version;
	const url = `https://github.com/rebase-foundation/precrypt/releases/download/v${ version }/precrypt-${ platform }.tar.gz`;
	return new Binary(url, { name: 'precrypt' });
}

module.exports = getBinary;