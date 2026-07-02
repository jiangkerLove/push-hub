# 构建管理端静态资源
FROM node:22-alpine AS build

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY index.html ./
COPY tsconfig.json tsconfig.app.json tsconfig.node.json ./
COPY vite.config.ts ./
COPY src ./src

RUN npm run build

# 用 nginx 托管，并通过环境变量反代后端 API
FROM nginx:1.27-alpine

COPY nginx/default.conf.template /etc/nginx/templates/default.conf.template
COPY --from=build /app/dist /usr/share/nginx/html

# 默认指向同机 Docker 网络外的本机 server；compose 中可改为 http://push-hub-server:3000
ENV API_UPSTREAM=http://host.docker.internal:3000

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -qO- http://127.0.0.1/health || exit 1
