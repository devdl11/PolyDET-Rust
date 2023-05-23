docker run --name mysql -d \
    -p 3306:3306 \
    -e MYSQL_ROOT_PASSWORD=polydetdb \
    --restart unless-stopped \
    mysql:8