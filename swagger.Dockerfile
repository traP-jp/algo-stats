FROM swaggerapi/swagger-ui:v5.3.1
COPY openapi.yaml /openapi.yaml
ENV SWAGGER_JSON=/openapi.yaml
