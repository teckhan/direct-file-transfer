const fs = require('fs')
const fastify = require('fastify')
const {
  downloadablesStore,
  downloadStatsStore,
  downloadStatsActions
} = require('../store')

const initDownloadStream = (request, downloadable) => {
  const stream = fs.createReadStream(downloadable?.path)
  let uploadedSize = 0
  stream.on('data', buffer => {
    downloadStatsStore.dispatch(
      downloadStatsActions.add({
        id: request.id,
        ip:
          request.headers['x-forwarded-for'] ||
          request.raw.connection.remoteAddress,
        uploadedSize: (uploadedSize += buffer.length),
        downloadable
      })
    )
  })
  stream.on('end', () => {
    downloadStatsStore.dispatch(downloadStatsActions.remove(request.id))
  })
  return stream
}

const app = fastify()

app.get('/', async (request, reply) => ({
  data: downloadablesStore.getState().downloadables.map(downloadable => {
    const { path, ...rest } = downloadable
    return rest
  })
}))

app.get('/dl', async (request, reply) => {
  // TODO: multiple files download
  const downloadable = downloadablesStore.getState().downloadables[0]

  const stream = initDownloadStream(request, downloadable)
  // const stream2 = initDownloadStream(
  //   request,
  //   downloadablesStore.getState().downloadables[1]
  // )
  reply.header(
    'Content-disposition',
    `attachment; filename=${downloadable?.name}`
  )
  reply.header('Content-length', downloadable?.size)
  reply.type(downloadable?.type)
  reply.send(stream)
  // reply.send(stream2)
})

const start = async (port = 3000) => {
  await app.listen(port, '0.0.0.0').catch(console.log)

  console.log(`Server started at port: ${port}`)

  return app
}

module.exports = start
