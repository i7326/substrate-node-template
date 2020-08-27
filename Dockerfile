FROM paritytech/substrate-playground-template-base:sha-ac4bc53

ADD . /var/www/contact-tracing-node

WORKDIR /var/www/contact-tracing-node

RUN ./target/release/contact-tracing-node purge-chain --dev -y

CMD ./target/release/contact-tracing-node --dev --ws-external