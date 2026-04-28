# Build stage
FROM node:20-bookworm-slim as builder

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm config set registry https://registry.npmmirror.com && npm install --legacy-peer-deps

# Copy source code
COPY . .

# Set API URL for build
ENV VITE_API_URL=http://localhost:8080

# Build the app
RUN npm run build

# Production stage
FROM node:20-bookworm-slim

WORKDIR /app

# Copy built files
COPY --from=builder /app/build ./build
COPY --from=builder /app/package*.json ./

# Install production dependencies only
RUN npm config set registry https://registry.npmmirror.com && npm ci --only=production --legacy-peer-deps

# Expose port
EXPOSE 5173

# Start the app
CMD ["node", "build"]
