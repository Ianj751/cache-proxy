# docker run -d --name redis -p 6379:6379 redis:latest
docker run --hostname=faa9ab778edc --env=PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin --env=REDIS_DOWNLOAD_URL=https://github.com/redis/redis/archive/refs/tags/8.0.2.tar.gz --env=REDIS_DOWNLOAD_SHA=caf3c0069f06fc84c5153bd2a348b204c578de80490c73857bee01d9b5d7401f --volume=/data --network=bridge --workdir=/data -p 6379:6379 --restart=no --runtime=runc -d redis:8.0.2

