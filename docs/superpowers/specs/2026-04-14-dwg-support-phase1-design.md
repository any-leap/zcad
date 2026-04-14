# plangen DWG 支持 · Phase 1 设计

**日期：** 2026-04-14
**状态：** 设计确认中
**关联项目：** `plangen`（消费方）、`zcad`（sidecar 归属方）

---

## 1. 背景与目标

### 1.1 现状

- `plangen`（TanStack Start + React + bun）当前用 `dxf-parser` 在**浏览器端**读 DXF，只抽取 TEXT / MTEXT / ATTRIB 文字实体喂给下游分析流程（`src/lib/drawing-analysis/`），未做几何渲染
- 需求方希望 plangen 直接能读 `.dwg`。当前绕法是"让用户在 CAD 里 `dxfout` 一下"——摩擦大
- DWG 是 Autodesk 私有二进制格式，版本多且仅部分公开。LibreDWG 只支持到 2018 格式，AutoCAD 2025+ 的 AC1036 新格式无法读取
- ODA File Converter（ODA 官方免费闭源工具）跟进所有新格式，包括 AC1036

### 1.2 长期目标与分阶段

- **Phase 1（本 spec 范围）：** plangen 接 ODA File Converter，最快路径获得 DWG 读取能力，包含 AC1036
- **Phase 2（独立 spec，后续）：** 在 zcad 里用 rust 写原生 DWG 解析器，替换 ODA FC，去除闭源依赖

本 spec 只描述 Phase 1。Phase 2 的存在影响 Phase 1 的接口设计——必须保证替换时对 plangen 无感。

### 1.3 Phase 1 目标

1. plangen 用户上传 `.dwg` 文件（任意 2000~2025 版本）后，抽取出的文字能进入现有分析流程
2. 引入的架构在 Phase 2 中原地可替换，plangen 侧代码零改动
3. plangen 原有 `.dxf` 路径零退化、零性能影响

### 1.4 非目标（Phase 1 不做）

- 在 plangen 里渲染/显示 DWG 图形
- 抽取尺寸/图层/块定义等非文字实体（未来可在同一份 DXF 上增量做，不影响本 spec）
- 支持 Windows / macOS 的原生部署路径（仅 docker）
- DXF → DWG 反向转换
- sidecar 和 plangen 之间的认证、限流、指标收集
- 内容哈希缓存

---

## 2. 架构总览

### 2.1 三个组件

```
┌──────────────────┐   upload .dwg    ┌──────────────────┐
│   plangen 前端   │ ───────────────> │  plangen server  │
│ (DrawingAnalyzer)│                  │  /api/convert-dwg│
└──────────────────┘                  └────────┬─────────┘
                                               │ HTTP POST
                                               │ multipart/form-data
                                               ▼
                                      ┌──────────────────┐
                                      │ zcad-dwg-service │
                                      │  (rust + axum)   │
                                      │                  │
                                      │  DwgConverter    │
                                      │  ├── OdaFcConv   │ Phase 1
                                      │  └── NativeConv  │ Phase 2 占位
                                      └────────┬─────────┘
                                               │ 临时文件 + shell-out
                                               ▼
                                      ┌──────────────────┐
                                      │  xvfb-run        │
                                      │  ODAFileConverter│
                                      └──────────────────┘
```

### 2.2 数据流

1. 用户在 `DrawingAnalyzer` 上传 `.dwg`
2. 前端按扩展名分流，调用 `extractDwgText(file)`
3. `extractDwgText` POST 到 `/api/convert-dwg`（TanStack Start API route）
4. API route 做前置校验（大小、扩展名），代理到 `http://dwg-converter:8080/convert`
5. sidecar：
   1. 写 DWG 字节到临时目录
   2. `xvfb-run -a ODAFileConverter <tmp_in> <tmp_out> ACAD2018 DXF 0 1 *.dwg`
   3. 读取生成的 DXF 内容为字符串
   4. 返回 DXF 文本，清理临时目录
6. plangen server 把 DXF 字符串传回前端
7. 前端复用 `extractDxfTextFromString(dxf)`，下游分析流程不变

### 2.3 Phase 1 → Phase 2 替换缝

sidecar 内部 `trait DwgConverter`：

```rust
#[async_trait]
pub trait DwgConverter: Send + Sync {
    async fn convert(&self, dwg: Bytes) -> Result<String, ConvertError>;
}
```

- Phase 1 唯一实现 `OdaFcConverter`（shell out）
- Phase 2 新增 `NativeConverter`（走 `zcad-file` 原生 rust 解析）
- 切换方式：环境变量 `ZCAD_DWG_BACKEND=oda|native`，默认 Phase 1 为 `oda`，Phase 2 落地后改为 `native`
- 对外 HTTP 契约（端点、输入、输出格式）不变
- plangen 完全感知不到

---

## 3. API 契约

### 3.1 `POST /convert`

**请求：**
- `Content-Type: multipart/form-data`
- 字段：`file`（DWG 二进制）

**成功响应：**
- `200 OK`
- `Content-Type: application/vnd.dxf; charset=utf-8`
- Body：DXF 文本（ACAD2018 版本）

**错误响应：**

| 状态码 | 场景 |
|---|---|
| 400 | 非 multipart / 缺 `file` 字段 |
| 413 | 文件超过 50 MB 上限 |
| 415 | 内容不是有效 DWG（magic bytes 校验失败） |
| 422 | ODA FC 退出非零（文件损坏、加密、格式不支持） |
| 500 | sidecar 内部错误（ODA FC 不存在、tmp 失败等） |
| 504 | 转换超时（默认 60 秒） |

错误响应 Body：`application/json`，结构 `{ "error": "<code>", "message": "<human readable>" }`

### 3.2 `GET /healthz`

- `200 OK`，body `"ok"`
- 用于 docker healthcheck 和 `depends_on: { condition: service_healthy }`

### 3.3 为什么返回 DXF 文本而非结构化 JSON

- plangen 已有成熟的 `dxf-parser`，零改动复用
- 未来 plangen 需要抽尺寸/图层时，仍在同一份 DXF 上做，sidecar 不用改
- Phase 2 接原生 rust 解析器时也能先输出 DXF，或未来再加第二个返回结构化数据的端点
- 代价：内网多传 1~10 MB 字符串，忽略不计

---

## 4. `zcad-dwg-service` 详细设计

### 4.1 crate 布局

```
zcad/crates/zcad-dwg-service/
├── Cargo.toml
├── Dockerfile
├── src/
│   ├── main.rs          # axum bootstrap、tracing 初始化、env 读取
│   ├── routes.rs        # POST /convert、GET /healthz
│   ├── converter.rs     # trait DwgConverter、ConvertError
│   ├── oda_fc.rs        # Phase 1: OdaFcConverter (shell-out)
│   └── native.rs        # Phase 2 占位：NativeConverter { unimplemented!() }
└── tests/
    ├── fixtures/
    │   ├── sample_2018.dwg
    │   ├── sample_2025.dwg
    │   └── corrupt.dwg
    └── convert_integration.rs
```

### 4.2 依赖（`Cargo.toml`）

- `axum` + `tokio` + `tower` + `tower-http`（multipart、body limit）
- `bytes`
- `async-trait`
- `tracing` + `tracing-subscriber`
- `tempfile`
- `thiserror`
- `serde` + `serde_json`

不依赖 `zcad-file` / `zcad-core` —— Phase 1 sidecar 是纯 shell-out，跟主 workspace 解耦；Phase 2 才会加 `zcad-file` 依赖。

### 4.3 配置（环境变量）

| 变量 | 默认值 | 说明 |
|---|---|---|
| `PORT` | `8080` | HTTP 监听端口 |
| `ZCAD_DWG_BACKEND` | `oda` | `oda` 或 `native`；Phase 1 只 `oda` 可用 |
| `ZCAD_DWG_MAX_BYTES` | `52428800`（50 MB） | 请求 body 上限 |
| `ZCAD_DWG_TIMEOUT_SECS` | `60` | 单次转换超时 |
| `ZCAD_DWG_ODA_BIN` | `ODAFileConverter` | ODA FC 可执行文件路径 |
| `RUST_LOG` | `info` | tracing 级别 |

### 4.4 `OdaFcConverter` 关键逻辑

```rust
impl DwgConverter for OdaFcConverter {
    async fn convert(&self, dwg: Bytes) -> Result<String, ConvertError> {
        // 1. 校验 magic bytes (前 6 字节 "AC" + 版本号)
        // 2. tempfile::tempdir() 建 in/ 和 out/
        // 3. fs::write("in/input.dwg", dwg)
        // 4. tokio::process::Command:
        //    "xvfb-run -a ODAFileConverter <in> <out> ACAD2018 DXF 0 1 *.dwg"
        //    用 tokio::time::timeout 包住
        // 5. 非零退出 → ConvertError::OdaFailed { stderr }
        // 6. 读 out/input.dxf
        // 7. tempdir drop 时自动清理
    }
}
```

**坑与对策：**

- **ODA FC 只吃目录不吃单文件** → 用 `tempfile::tempdir` 建 in/out 两个临时目录
- **xvfb 必须 `-a`**（自动选空闲 display） → 支持并发调用
- **ODA FC 启动开销 ~1s，典型总耗时 2~5s** → 超时默认 60s 留充分余量
- **ODA FC 进程有时卡住** → 必须用 tokio timeout 包住并主动 kill
- **临时目录必须在子进程退出后才 drop** → `.await` 完整 Command future 再读文件

### 4.5 并发模型

- axum tower 默认并发足够（单租户部署）
- 不做进程池、不做队列
- ODA FC 每次独立进程，xvfb `-a` 保证 display 不冲突
- 只靠 `ZCAD_DWG_MAX_BYTES` 和 OS 级资源限制防过载

---

## 5. Docker 与部署

### 5.1 Dockerfile（`crates/zcad-dwg-service/Dockerfile`）

```dockerfile
FROM rust:1.83 AS builder
WORKDIR /build
COPY . .
RUN cargo build --release -p zcad-dwg-service

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y --no-install-recommends \
      xvfb libxkbcommon0 libglib2.0-0 libgl1 libfontconfig1 \
      libsm6 libxrender1 libxi6 ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*

# .deb 不入 repo；构建时从 CI secret 里传下载 URL
ARG ODA_DEB_URL
RUN curl -fsSL "$ODA_DEB_URL" -o /tmp/oda.deb && \
    dpkg -i /tmp/oda.deb || apt-get install -fy && \
    rm /tmp/oda.deb

COPY --from=builder /build/target/release/zcad-dwg-service /usr/local/bin/

ENV ZCAD_DWG_BACKEND=oda \
    ZCAD_DWG_MAX_BYTES=52428800 \
    ZCAD_DWG_TIMEOUT_SECS=60 \
    PORT=8080 \
    RUST_LOG=info

HEALTHCHECK --interval=30s --timeout=5s \
  CMD curl -f http://localhost:8080/healthz || exit 1

CMD ["xvfb-run", "-a", "/usr/local/bin/zcad-dwg-service"]
```

### 5.2 ODA File Converter .deb 获取

- **不放进 repo**（分发条款限制）
- 在 `any-leap` GitHub org 的 Actions secrets 里存 `ODA_DEB_URL`（ODA 官方 .deb 的直链，需要账号下载后放到私有对象存储或 release asset）
- GitHub Actions build workflow 里 `--build-arg ODA_DEB_URL=$ODA_DEB_URL`
- 本地开发：约定放到 `~/.cache/oda/ODAFileConverter_QT6_lnxX64_8.3dll_26.4.deb`（或类似路径），`docker build --build-arg ODA_DEB_URL=file:///...`

### 5.3 镜像发布

- Registry: `ghcr.io/any-leap/zcad-dwg-service`
- Tags: `latest`（main 分支）、`<git-sha>`（每次 push）、`v<version>`（tag 时）
- GitHub Actions workflow 位置：`zcad/.github/workflows/dwg-service.yml`

### 5.4 plangen 的 `docker-compose.yml` 变更

```yaml
services:
  dwg-converter:
    image: ghcr.io/any-leap/zcad-dwg-service:latest
    container_name: dwg-converter
    restart: unless-stopped
    networks:
      - plangen-internal
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 5s
      retries: 3

  plangen:
    # ...（原有配置）
    environment:
      # ...（原有）
      - DWG_CONVERTER_URL=http://dwg-converter:8080
    networks:
      - proxy
      - plangen-internal
    depends_on:
      dwg-converter:
        condition: service_healthy

networks:
  proxy:
    external: true
  plangen-internal: {}
```

### 5.5 部署步骤

1. `docker compose pull`（拉新版 sidecar）
2. `docker compose up -d`
3. 等 `dwg-converter` 健康检查通过
4. plangen 启动后自动 `depends_on` 通过

---

## 6. plangen 改动清单

### 6.1 新增文件

1. **`src/server/dwg-converter-client.ts`**
   - 从 `process.env.DWG_CONVERTER_URL` 读地址
   - 导出 `convertDwgToDxf(file: File | Blob): Promise<string>`
   - 封装 multipart 请求、HTTP 错误码到错误类型的映射

2. **`src/routes/api/convert-dwg.ts`**（TanStack Start API route）
   - 接收前端上传 → 前置校验（扩展名、大小）→ 代理到 sidecar → 返 DXF 文本
   - 简单内存 token bucket 限流（防本地爆量），单租户无需跨实例协调

3. **`src/lib/drawing-analysis/dwg-reader.ts`**
   ```ts
   import { convertDwgToDxf } from "@/server/dwg-converter-client"
   import { extractDxfTextFromString } from "./dxf-reader"

   export async function extractDwgText(file: File | Blob): Promise<string> {
     const dxfText = await convertDwgToDxf(file)
     return extractDxfTextFromString(dxfText)
   }
   ```

### 6.2 修改现有文件

4. **`src/lib/drawing-analysis/dxf-reader.ts`**
   - 把"字符串 → 文字"核心抽成独立导出函数 `extractDxfTextFromString(dxf: string): string`
   - 现有 `extractDxfText(file)` 内部改为 `extractDxfTextFromString(await file.text())`
   - 这是对现有代码唯一的修改

5. **`src/components/editor/DrawingAnalyzer.tsx`**
   - `<input accept>` 加 `.dwg`
   - 文件类型分流：`.dxf` → `extractDxfText`；`.dwg` → `extractDwgText`
   - loading 文案区分："解析 DXF..." vs "转换 DWG 中（预计 2~5 秒）..."

6. **`docker-compose.yml`** & **`.env.example`**
   - 加 `dwg-converter` service
   - plangen service 加 `DWG_CONVERTER_URL` 环境变量 + 内网
   - `.env.example` 加 `DWG_CONVERTER_URL=http://dwg-converter:8080` 示例

### 6.3 完全不动

- `dxf-parser` 依赖和版本
- `drawing-analysis/` 下游所有代码
- 数据库 schema / Drizzle migrations
- 任何 UI 组件库
- 现有 DXF 路径的行为

---

## 7. 错误处理与 UX

### 7.1 错误码映射表

| sidecar 状态码 | plangen 前端展示 | 日志级别 |
|---|---|---|
| 200 | 继续分析流程 | debug |
| 413 | "DWG 文件超过 50MB 上限" | info |
| 415 | "文件不是有效的 DWG" | info |
| 422 | "DWG 无法转换（可能损坏、加密或使用了不支持的特性）。建议在 CAD 里另存为 DXF 后重试。" | warn |
| 504 | "转换超时，文件可能过于复杂。建议另存为 DXF。" | warn |
| 5xx / 网络错 | "转换服务不可用，请稍后重试或导出 DXF。" | error |

### 7.2 Fallback 原则

所有 422/504/5xx 错误提示中都包含"另存为 DXF"的后备路径。即使 sidecar 对某文件失败，用户也能不改流程继续工作。

### 7.3 Loading 状态

DWG 转换典型 2~5 秒，必须有明确 loading UI。DXF 本地解析是毫秒级，无需改 loading。

---

## 8. 测试策略

### 8.1 `zcad-dwg-service`

**单元测试：**
- `OdaFcConverter`：mock `std::process::Command`，验证参数组装、超时、退出码映射、临时目录清理
- `routes`：用 `axum::Router::into_make_service` 启本地 listener，mock `DwgConverter` 返固定字符串，验证响应头和状态码

**集成测试（`tests/convert_integration.rs`，`--features integration` 下跑）：**
- 启真实 axum server
- POST 三个 fixture：
  - `sample_2018.dwg` → 期望 200，响应体包含已知字符串
  - `sample_2025.dwg` → 期望 200（验证 ODA FC 覆盖新格式）
  - `corrupt.dwg`（截断字节） → 期望 422
- 前提：测试环境有 ODA FC（CI 里用 build 好的镜像作为 test runner）

### 8.2 plangen

- 单元：mock `/api/convert-dwg`，验证 `extractDwgText` 正常路径和错误映射
- e2e（vitest）：上传小 DWG fixture，断言输出文字包含预期内容
- CI 里 e2e 启 docker-compose（plangen + dwg-converter）跑

---

## 9. 验收标准（Phase 1 "完成"的定义）

1. 用户在 plangen 的 DrawingAnalyzer 上传 2018 格式 DWG，5 秒内看到抽取出的文字
2. 上传 AutoCAD 2025 原生保存的 DWG（AC1036），同样成功
3. 上传损坏 DWG，看到友好错误提示（非 500 stack trace）
4. 初次 `docker compose up -d` 从启动到服务 ready 不超过 60 秒
5. sidecar 被 kill 后 plangen 的 DXF 老路径继续工作
6. `trait DwgConverter` 存在；`NativeConverter` 结构体和 impl 占位存在（为 Phase 2 铺路）
7. `zcad` repo 的 README 里新增 sidecar 的最小部署说明和 `ODA_DEB_URL` 获取方式
8. plangen repo 的 README 更新 docker-compose 新 service 的说明

---

## 10. 风险与缓解

| 风险 | 缓解 |
|---|---|
| ODA FC 分发条款变化 / 断链 | 把 .deb 镜像缓存到 any-leap 的私有对象存储，CI 指向镜像而非官方直链 |
| ODA FC 对某些 DWG 崩溃或卡住 | tokio timeout 主动 kill；返 422 让用户走 DXF 后备路径 |
| xvfb 在 docker 内不稳定 | `xvfb-run -a` 每次起独立 display；失败即视为 500 重试 |
| 镜像 ~800MB 过大 | 本 phase 接受；Phase 2 替换 ODA FC 后可降到 ~50MB |
| ODA FC 版本老化 | GitHub Actions 定期重建镜像；ODA_DEB_URL 可升级 |
| plangen 部署机无法访问 ghcr.io | 提前验证；必要时推到用户自有 registry |

---

## 11. 后续步骤

本 spec 确认后：

1. 用 `writing-plans` skill 将本设计转成实施 plan（拆分原子任务、依赖顺序、每步验收）
2. Phase 1 实施完成、上线稳定后，再单独 brainstorm Phase 2（zcad 原生 rust DWG 解析器）
3. Phase 2 的入口点是替换 `zcad-dwg-service` 里 `NativeConverter` 的占位实现，plangen 不参与

---

**签署：** 此设计需用户确认后才进入 writing-plans 阶段。
