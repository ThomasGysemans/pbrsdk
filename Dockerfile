FROM debian

RUN apt-get update \
    && apt-get install -y curl unzip

RUN mkdir -p /pb_data /work

WORKDIR /work
VOLUME ["/pb_data"]

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh \
    && curl -L https://github.com/pocketbase/pocketbase/releases/download/v0.36.1/pocketbase_0.36.1_linux_arm64.zip -o /tmp/pocketbase.zip \
    && unzip /tmp/pocketbase.zip -d /tmp \
    && mv /tmp/pocketbase /usr/local/bin/pocketbase \
    && chmod +x /usr/local/bin/pocketbase

EXPOSE 8090
ENTRYPOINT ["/entrypoint.sh"]
CMD [""]