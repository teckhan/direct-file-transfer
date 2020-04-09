const express = require('express');
const helmet = require('helmet');
const fs = require('fs');
const path = require('path');
const ip = require("ip");
const fileUpload = require('express-fileupload');
const app = express();
const port = 80;
const ipLocal = ip.address() + (port != '80' ? ':' + port : '');

const pathFileDefault = path.join(require('os').homedir(), 'Desktop');;
let filenames = [
	//{ name: 'file.txt' },
];

app
	.set('trust proxy', true)
	.use(helmet())
	.use('/assets', express.static(path.join(__dirname, './assets')))
	.use(express.urlencoded({extended: true}))
	.use(express.json())
	.use(fileUpload())
	.engine('html', require('ejs').renderFile)

	.get('/', function (req, res) {
		let isHost = req.ip.indexOf(ip.address()) > -1 || ['127.0.0.1', '::ffff:127.0.0.1', '::1'].indexOf(req.ip) > -1;
		// isHost = false;

		res.setHeader("Cache-Control", "no-cache, must-revalidate")
		if (isHost)
			res.render(path.join(__dirname, './transfer.html'), { ip: ipLocal }); // prepare & transfer files to owner's device
		else
			res.render(path.join(__dirname, './index.html'), { download: filenames.length > 0 }); // prepare files to transfer from owner's device
	})

	// download files from owner's device
	.get('/dl', function (req, res) {
		if (filenames.length > 0) {
			let filesArray = filenames;

			filesArray.map(function (file) {
				if (typeof file.path === 'undefined')
					file.path = pathFileDefault;

				if (file.path.indexOf(file.name) < 0)
					file.path = path.join(file.path, file.name);

				fs.stat(file.path, function (err, stat) {
					if (err !== null && err.code === 'ENOENT') {
						res.end(file.name + ' is missing.')
					}
				});
			});

			if (filesArray.length > 1)
				res.zip(filesArray);
			else
				res.download(filesArray[0].path, filesArray[0].name);
		} else {
			res.end('No file to be downloaded at the moment.');
		}
	})
	.get('/*?', function(req, res) {
		res.redirect('/');
	})

	// transfer files to owner's device
	.post('/fileupload', function(req, res) {
		if (!req.files)
			return res.status(400).send('No files were uploaded.');

		files = req.files.filetoupload;
		arr_uploaded = [];

		function save(obj) {
			obj.mv(path.join(pathFileDefault, obj.name), function(err) {
				if (err)
					return res.status(500).send(err);
			});
		}

		if (files.length === undefined) {
			save(files);
			res.end('{"success": true, "text": "Uploaded Files: ' + files.name + '"}');
		}
		else {
			for (var i = 0, len = files.length; i < len; i++) {
				save(files[i])
				arr_uploaded.push(files[i].name);
				if (arr_uploaded.length == len) {
					res.end('{"success": true, "text": "Uploaded Files: ' + arr_uploaded + '"}');
				}
			}
		}
	})

	// prepare files to transfer from owner's device
	.post('/transferEndPoint', function (req, res) {
		let filesTemp = req.body.files;

		if (typeof filesTemp === 'object' && filesTemp.length > 0) {
			filenames.length = 0;
			filesTemp.map(function (filename) { filenames.push(filename) });
			res.end(JSON.stringify(
				{
					"success": true,
					"text": "Ready to be downloaded: " + (filenames.map(function (file) { return file.name }).join(', '))
				}
			));
		}
	})
	.listen(port, function () {
		console.log( ip.address() + (this.address().port != '80' ? ':' + this.address().port : '') );
	})

module.exports = {
	ip: ipLocal
}