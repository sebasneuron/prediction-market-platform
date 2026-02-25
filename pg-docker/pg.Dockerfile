FROM postgres:16-bookworm

ARG POSTGRES_DB=postgres

RUN echo "POSTGRES_DB: $POSTGRES_DB"

ENV POSTGRES_DB=${POSTGRES_DB}

RUN apt-get update && \
    apt-get -y install postgresql-16-cron && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN echo "shared_preload_libraries='pg_cron'" >> /usr/share/postgresql/postgresql.conf.sample

# Change permissions of the postgresql.conf.sample file
RUN chown postgres:postgres /usr/share/postgresql/postgresql.conf.sample && \
    chmod 664 /usr/share/postgresql/postgresql.conf.sample

COPY init-scripts/ /docker-entrypoint-initdb.d/
COPY entrypoint.sh /usr/local/bin/entrypoint.sh

RUN chmod +x /usr/local/bin/entrypoint.sh

USER postgres
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["postgres"]