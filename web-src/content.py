# Page content for the AIPCMaster site. One entry per page, both languages.
#
# Kept as data rather than as twenty-two hand-written HTML files so the nav, the footer and the
# <head> exist once (in generate.py) and every page is guaranteed to carry them. Content is
# sourced from the PRD (§1, §3, §4, §7, §8), the ERD (§5) and the software document (§1, §2).

NAV = {
    "en": [
        ("Features", "index.html#features"),
        ("How it works", "index.html#how"),
        ("Pricing", "pricing.html"),
        ("Business", "business.html"),
        ("Developers", "developers.html"),
        ("Docs", "docs.html"),
    ],
    "zh": [
        ("核心能力", "index.zh.html#features"),
        ("工作原理", "index.zh.html#how"),
        ("价格", "pricing.zh.html"),
        ("企业版", "business.zh.html"),
        ("开发者", "developers.zh.html"),
        ("文档中心", "docs.zh.html"),
    ],
}

CTA = {"en": ("Download", "download.html"), "zh": ("下载", "download.zh.html")}
OTHER = {"en": ("中文", "index.zh.html"), "zh": ("EN", "index.html")}

# slug -> {lang: (title, description, body_html)}
PAGES = {}

# ── download ───────────────────────────────────────────────────────────────────

PAGES["download"] = {
    "en": (
        "Download",
        "Download AIPCMaster for Windows. The V1.0 build ships in Q4 2026 — join the list and we will send it the day it does.",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">DOWNLOAD</span>
    <h2>Windows first, macOS next</h2>
    <p class="lead">V1.0 is in build and ships in Q4 2026. macOS follows in V1.5.</p>
    <div class="grid-2">
      <div class="card">
        <h3>Windows 10 / 11 &nbsp;<span class="badge">V1.0 · Q4 2026</span></h3>
        <p>x64. The installer signs itself and checks its own signature before it writes anything.</p>
        <p style="margin-top:14px"><a class="btn btn-primary" href="trial.html">Join the list</a></p>
      </div>
      <div class="card">
        <h3>macOS &nbsp;<span class="badge">V1.5 · 2027 Q2</span></h3>
        <p>SwiftUI client over the same Rust core. Predictive maintenance lands with it.</p>
        <p style="margin-top:14px"><a class="btn btn-ghost" href="trial.html">Notify me</a></p>
      </div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">REQUIREMENTS</span>
      <h2>What it needs</h2>
    </div>
    <div class="grid-4">
      <div class="card"><h3>System</h3><p>Windows 10 21H2 or Windows 11, x64.</p></div>
      <div class="card"><h3>Memory</h3><p>4 GB minimum. 8 GB if you want the local model resident.</p></div>
      <div class="card"><h3>Disk</h3><p>2 GB for the client; more for model bundles if you keep several.</p></div>
      <div class="card"><h3>Acceleration</h3><p>NPU or GPU helps, and is optional. It falls back to CPU.</p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">INSTALL</span>
      <h2>Three steps</h2>
    </div>
    <div class="grid-3 flow">
      <div class="card"><h3>Install</h3><p>Run the signed installer. It creates a restore point before its first change.</p></div>
      <div class="card"><h3>Sign in</h3><p>Email, phone, or Google / Apple / WeChat depending on your region.</p></div>
      <div class="card"><h3>Diagnose</h3><p>The 14-day trial starts on its own. Run one diagnostic and read the report.</p></div>
    </div>
  </div>
</section>
""",
    ),
    "zh": (
        "下载",
        "下载 AI电脑大师 Windows 版。V1.0 将于 2026 Q4 发布，留下邮箱，发布当天通知你。",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">下载</span>
    <h2>先 Windows，再 macOS</h2>
    <p class="lead">V1.0 正在开发中，2026 Q4 发布。macOS 随 V1.5 跟进。</p>
    <div class="grid-2">
      <div class="card">
        <h3>Windows 10 / 11 &nbsp;<span class="badge">V1.0 · 2026 Q4</span></h3>
        <p>x64。安装包自带签名，写入任何文件之前先校验自己的签名。</p>
        <p style="margin-top:14px"><a class="btn btn-primary" href="trial.zh.html">加入通知列表</a></p>
      </div>
      <div class="card">
        <h3>macOS &nbsp;<span class="badge">V1.5 · 2027 Q2</span></h3>
        <p>SwiftUI 客户端，共用同一套 Rust 核心。预测性维护同期上线。</p>
        <p style="margin-top:14px"><a class="btn btn-ghost" href="trial.zh.html">发布时通知我</a></p>
      </div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">系统要求</span>
      <h2>需要什么配置</h2>
    </div>
    <div class="grid-4">
      <div class="card"><h3>系统</h3><p>Windows 10 21H2 或 Windows 11，x64。</p></div>
      <div class="card"><h3>内存</h3><p>最低 4 GB；希望本地模型常驻则建议 8 GB。</p></div>
      <div class="card"><h3>磁盘</h3><p>客户端 2 GB；若保留多套模型包需要更多。</p></div>
      <div class="card"><h3>加速</h3><p>NPU 或 GPU 更好，但非必需，可回退到 CPU。</p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">安装</span>
      <h2>三步</h2>
    </div>
    <div class="grid-3 flow">
      <div class="card"><h3>安装</h3><p>运行已签名的安装包。首次改动之前，它会自动创建一个还原点。</p></div>
      <div class="card"><h3>登录</h3><p>邮箱、手机号，或按地区启用 Google / Apple / 微信登录。</p></div>
      <div class="card"><h3>诊断</h3><p>14 天试用自动开始。跑一次诊断，看报告。</p></div>
    </div>
  </div>
</section>
""",
    ),
}

# ── pricing ────────────────────────────────────────────────────────────────────

PAGES["pricing"] = {
    "en": (
        "Pricing",
        "AIPCMaster pricing: 14-day free trial, Personal ¥199/year, Family ¥349/year for 5 devices, Business ¥99/device/year.",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">PRICING</span>
    <h2>Start free. Pay when it has earned it.</h2>
    <p class="lead">Every new account gets the full product for 14 days. After that the free tier keeps basic diagnostics.</p>
    <div class="price-grid">
      <div class="tier">
        <div class="name">Trial</div>
        <div class="amount">Free <small>14 days</small></div>
        <ul><li>Full diagnostics</li><li>Basic optimisation</li><li>No card required</li></ul>
      </div>
      <div class="tier feature">
        <div class="name">Personal<span class="badge">POPULAR</span></div>
        <div class="amount">¥199 <small>CNY / year</small></div>
        <ul><li>Everything in Trial, permanently</li><li>Automatic optimisation</li><li>Predictive maintenance</li><li>Privacy controls</li></ul>
      </div>
      <div class="tier">
        <div class="name">Family</div>
        <div class="amount">¥349 <small>CNY / year</small></div>
        <ul><li>Everything in Personal</li><li>Up to 5 devices</li><li>Shared device health report</li></ul>
      </div>
      <div class="tier">
        <div class="name">Business</div>
        <div class="amount">¥99 <small>CNY / device / year</small></div>
        <ul><li>Central management console</li><li>Batch deployment</li><li>API integration</li><li>Priority support</li></ul>
      </div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">COMPARE</span>
      <h2>What each tier includes</h2>
    </div>
    <table class="tbl">
      <thead><tr><th>Capability</th><th>Trial</th><th>Personal</th><th>Family</th><th>Business</th></tr></thead>
      <tbody>
        <tr><td>System monitoring</td><td>✓</td><td>✓</td><td>✓</td><td>✓</td></tr>
        <tr><td>Smart diagnostics &amp; reports</td><td>✓</td><td>✓</td><td>✓</td><td>✓</td></tr>
        <tr><td>Automatic optimisation</td><td>basic</td><td>✓</td><td>✓</td><td>✓</td></tr>
        <tr><td>Predictive maintenance</td><td>—</td><td>✓</td><td>✓</td><td>✓</td></tr>
        <tr><td>Devices per licence</td><td>1</td><td>1</td><td>5</td><td>per device</td></tr>
        <tr><td>Central console &amp; batch deploy</td><td>—</td><td>—</td><td>—</td><td>✓</td></tr>
        <tr><td>API integration</td><td>—</td><td>—</td><td>—</td><td>✓</td></tr>
        <tr><td>Support</td><td>community</td><td>email</td><td>email</td><td>priority</td></tr>
      </tbody>
    </table>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">PAYMENT</span><h2>How you pay</h2></div>
    <div class="grid-4">
      <div class="card"><h3>Alipay</h3><p>Scan and go.</p></div>
      <div class="card"><h3>WeChat Pay</h3><p>Same, from inside WeChat.</p></div>
      <div class="card"><h3>Bank card</h3><p>Visa and Mastercard.</p></div>
      <div class="card"><h3>Bank transfer</h3><p>For Business, with an invoice.</p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">FAQ</span><h2>Questions people actually ask</h2></div>
    <div class="grid-2">
      <div class="card"><h3>Does the trial need a card?</h3><p>No. It starts when you sign in and simply ends.</p></div>
      <div class="card"><h3>What happens when it ends?</h3><p>The account drops to the free tier. Nothing is deleted.</p></div>
      <div class="card"><h3>Can I move a licence to a new PC?</h3><p>Yes. Unbind the old device and bind the new one; the count follows the licence, not the machine.</p></div>
      <div class="card"><h3>Refunds?</h3><p>Within 14 days of a charge, from the subscription page. No form to fill in.</p></div>
    </div>
  </div>
</section>
""",
    ),
    "zh": (
        "价格",
        "AI电脑大师价格：14 天免费试用，个人版 ¥199/年，家庭版 ¥349/年（5 台设备），企业版 ¥99/设备/年。",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">价格</span>
    <h2>先免费用。它值了再付。</h2>
    <p class="lead">每个新账号都有 14 天全功能试用。到期后，免费版保留基础诊断能力。</p>
    <div class="price-grid">
      <div class="tier">
        <div class="name">试用版</div>
        <div class="amount">免费 <small>14 天</small></div>
        <ul><li>全功能诊断</li><li>基础优化</li><li>无需绑定银行卡</li></ul>
      </div>
      <div class="tier feature">
        <div class="name">个人版<span class="badge">最受欢迎</span></div>
        <div class="amount">¥199 <small>/ 年</small></div>
        <ul><li>试用版全部能力，长期可用</li><li>自动优化</li><li>预测性维护</li><li>隐私与安全设置</li></ul>
      </div>
      <div class="tier">
        <div class="name">家庭版</div>
        <div class="amount">¥349 <small>/ 年</small></div>
        <ul><li>个人版全部权益</li><li>最多 5 台设备</li><li>设备健康报告共享</li></ul>
      </div>
      <div class="tier">
        <div class="name">企业版</div>
        <div class="amount">¥99 <small>/ 设备 / 年</small></div>
        <ul><li>集中管理控制台</li><li>批量部署</li><li>API 集成</li><li>优先支持</li></ul>
      </div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">对比</span><h2>各版本包含什么</h2></div>
    <table class="tbl">
      <thead><tr><th>能力</th><th>试用版</th><th>个人版</th><th>家庭版</th><th>企业版</th></tr></thead>
      <tbody>
        <tr><td>系统状态监控</td><td>✓</td><td>✓</td><td>✓</td><td>✓</td></tr>
        <tr><td>智能诊断与报告</td><td>✓</td><td>✓</td><td>✓</td><td>✓</td></tr>
        <tr><td>自动优化</td><td>基础</td><td>✓</td><td>✓</td><td>✓</td></tr>
        <tr><td>预测性维护</td><td>—</td><td>✓</td><td>✓</td><td>✓</td></tr>
        <tr><td>每份授权设备数</td><td>1</td><td>1</td><td>5</td><td>按设备</td></tr>
        <tr><td>集中控制台与批量部署</td><td>—</td><td>—</td><td>—</td><td>✓</td></tr>
        <tr><td>API 集成</td><td>—</td><td>—</td><td>—</td><td>✓</td></tr>
        <tr><td>支持</td><td>社区</td><td>邮件</td><td>邮件</td><td>优先</td></tr>
      </tbody>
    </table>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">支付</span><h2>怎么付款</h2></div>
    <div class="grid-4">
      <div class="card"><h3>支付宝</h3><p>扫码即付。</p></div>
      <div class="card"><h3>微信支付</h3><p>在微信内直接完成。</p></div>
      <div class="card"><h3>银行卡</h3><p>Visa 与 Mastercard。</p></div>
      <div class="card"><h3>企业转账</h3><p>企业版专用，可开具发票。</p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">常见问题</span><h2>大家真正会问的</h2></div>
    <div class="grid-2">
      <div class="card"><h3>试用需要绑卡吗？</h3><p>不需要。登录即开始，到期自动结束。</p></div>
      <div class="card"><h3>到期后会怎样？</h3><p>账号降级到免费版，数据不会被删除。</p></div>
      <div class="card"><h3>换电脑能转移授权吗？</h3><p>可以。解绑旧设备、绑定新设备即可；授权数跟着授权走，不跟着机器走。</p></div>
      <div class="card"><h3>可以退款吗？</h3><p>扣费后 14 天内，在订阅页面直接申请，无需填表。</p></div>
    </div>
  </div>
</section>
""",
    ),
}

# ── business ───────────────────────────────────────────────────────────────────

PAGES["business"] = {
    "en": (
        "Business",
        "AIPCMaster for IT teams: central console, batch deployment, API integration, audit logs and priority support, from ¥99 per device per year.",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">BUSINESS</span>
    <h2>Fleet diagnostics without the fleet of tools</h2>
    <p class="lead">For 10 to 500-person companies with no dedicated IT bench, and for IT departments past that.</p>
    <div class="grid-3">
      <div class="card"><h3>Central console</h3><p>Every device, its health, and what changed, in one place.</p></div>
      <div class="card"><h3>Batch deployment</h3><p>Roll the client out through your existing software distribution.</p></div>
      <div class="card"><h3>API integration</h3><p>Pull diagnostics and alerts into the systems you already run.</p></div>
      <div class="card"><h3>Audit log</h3><p>Who did what, on which device, and when. Exportable.</p></div>
      <div class="card"><h3>Role separation</h3><p>Owner, admin, operator, viewer — each with its own scope.</p></div>
      <div class="card"><h3>Priority support</h3><p>A named contact, not a queue.</p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">WHO IT FITS</span><h2>Where it lands</h2></div>
    <div class="grid-3">
      <div class="card"><h3>Small business</h3><p>10–500 people, no full-time IT. Diagnose before you call someone.</p></div>
      <div class="card"><h3>IT department</h3><p>500+, needing remote diagnosis, batch rollout and compliance evidence.</p></div>
      <div class="card"><h3>Managed service</h3><p>Run it across client sites under one console.</p></div>
    </div>
  </div>
</section>

<section class="cta">
  <div class="wrap">
    <h2>Talk to us about your fleet</h2>
    <p>Tell us how many devices and what you need to prove to whom. We will tell you straight whether it fits.</p>
    <a class="btn btn-primary" href="mailto:business@aipcmaster.com">business@aipcmaster.com</a>
    <a class="btn btn-ghost" href="mailto:sales@aipcmaster.com">sales@aipcmaster.com</a>
  </div>
</section>
""",
    ),
    "zh": (
        "企业版",
        "面向 IT 团队的 AI电脑大师：集中控制台、批量部署、API 集成、审计日志与优先支持，¥99/设备/年起。",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">企业版</span>
    <h2>管理一整个机队，不用管理一堆工具</h2>
    <p class="lead">适合 10–500 人、没有专职 IT 运维的中小企业，也适合规模更大的 IT 部门。</p>
    <div class="grid-3">
      <div class="card"><h3>集中管理控制台</h3><p>每台设备、它的健康度、以及发生了什么改动，都在一处。</p></div>
      <div class="card"><h3>批量部署</h3><p>走你现有的软件分发渠道，把客户端推下去。</p></div>
      <div class="card"><h3>API 集成</h3><p>把诊断结果与告警接入你已有的系统。</p></div>
      <div class="card"><h3>审计日志</h3><p>谁、在哪台设备、做了什么、什么时候。可导出。</p></div>
      <div class="card"><h3>角色分离</h3><p>所有者、管理员、操作员、只读，各自权限边界清晰。</p></div>
      <div class="card"><h3>优先支持</h3><p>对接具体的人，而不是排队。</p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">适用场景</span><h2>用在哪里</h2></div>
    <div class="grid-3">
      <div class="card"><h3>中小企业</h3><p>10–500 人、没有专职运维。先诊断，再决定要不要叫人。</p></div>
      <div class="card"><h3>企业 IT 部门</h3><p>500 人以上，需要远程诊断、批量部署与合规取证。</p></div>
      <div class="card"><h3>IT 服务商</h3><p>用一套控制台覆盖多个客户现场。</p></div>
    </div>
  </div>
</section>

<section class="cta">
  <div class="wrap">
    <h2>聊聊你的设备规模</h2>
    <p>告诉我们有多少台设备、需要向谁证明什么。合不合适，我们直说。</p>
    <a class="btn btn-primary" href="mailto:business@aipcmaster.com">business@aipcmaster.com</a>
    <a class="btn btn-ghost" href="mailto:sales@aipcmaster.com">sales@aipcmaster.com</a>
  </div>
</section>
""",
    ),
}

# ── developers ─────────────────────────────────────────────────────────────────

PAGES["developers"] = {
    "en": (
        "Developers",
        "The AIPCMaster API: REST and WebSocket/gRPC, OAuth2 / OIDC with JWT, described with OpenAPI. SDKs arrive with the Business edition.",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">DEVELOPERS</span>
    <h2>Read the fleet, programmatically</h2>
    <p class="lead">The API ships with the Business edition in V2.0. It is described with OpenAPI, so a client is a generator run away.</p>
    <div class="grid-3">
      <div class="card"><h3>REST + WebSocket / gRPC</h3><p>REST for state, a stream for what is changing now.</p></div>
      <div class="card"><h3>OAuth2 / OIDC + JWT</h3><p>Standard flows, scoped tokens, no bespoke auth to learn.</p></div>
      <div class="card"><h3>OpenAPI / Swagger</h3><p>The spec is the documentation. Generated clients are first-class.</p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">SURFACE</span><h2>What the API covers</h2></div>
    <div class="grid-4">
      <div class="card"><h3>Devices</h3><p>List, inspect, bind and unbind.</p></div>
      <div class="card"><h3>Diagnostics</h3><p>Run a session, fetch the report, follow its history.</p></div>
      <div class="card"><h3>Alerts</h3><p>Subscribe to thresholds and predictions.</p></div>
      <div class="card"><h3>Organisations</h3><p>Members, roles, API keys.</p></div>
    </div>
  </div>
</section>

<section class="cta">
  <div class="wrap">
    <h2>Want early access to the spec?</h2>
    <p>We will send the OpenAPI document and a sandbox key to the first people who ask.</p>
    <a class="btn btn-primary" href="mailto:dev@aipcmaster.com">dev@aipcmaster.com</a>
    <a class="btn btn-ghost" href="docs.html">Read the docs</a>
  </div>
</section>
""",
    ),
    "zh": (
        "开发者",
        "AI电脑大师 API：REST 与 WebSocket/gRPC，OAuth2 / OIDC + JWT，OpenAPI 描述。SDK 随企业版提供。",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">开发者</span>
    <h2>用程序读你的整个机队</h2>
    <p class="lead">API 随企业版在 V2.0 提供，用 OpenAPI 描述——生成一个客户端只差跑一次生成器。</p>
    <div class="grid-3">
      <div class="card"><h3>REST + WebSocket / gRPC</h3><p>REST 取状态，流式接口取"此刻正在变的东西"。</p></div>
      <div class="card"><h3>OAuth2 / OIDC + JWT</h3><p>标准流程、按范围授权，没有自创的认证方式要学。</p></div>
      <div class="card"><h3>OpenAPI / Swagger</h3><p>规范即文档。生成的客户端是一等公民。</p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">接口范围</span><h2>API 覆盖什么</h2></div>
    <div class="grid-4">
      <div class="card"><h3>设备</h3><p>列表、详情、绑定与解绑。</p></div>
      <div class="card"><h3>诊断</h3><p>发起会话、获取报告、追溯历史。</p></div>
      <div class="card"><h3>告警</h3><p>订阅阈值与预测事件。</p></div>
      <div class="card"><h3>组织</h3><p>成员、角色、API 密钥。</p></div>
    </div>
  </div>
</section>

<section class="cta">
  <div class="wrap">
    <h2>想先拿到规范文档？</h2>
    <p>我们会把 OpenAPI 文档和沙箱密钥发给最先来问的人。</p>
    <a class="btn btn-primary" href="mailto:dev@aipcmaster.com">dev@aipcmaster.com</a>
    <a class="btn btn-ghost" href="docs.zh.html">阅读文档</a>
  </div>
</section>
""",
    ),
}

# ── security ───────────────────────────────────────────────────────────────────

PAGES["security"] = {
    "en": (
        "Security",
        "How AIPCMaster is built to be safe: on-device processing, read-only by default, restore points before every change, signed updates and an audit log.",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">SECURITY</span>
    <h2>It can read your system. That is exactly why it is careful.</h2>
    <p class="lead">A tool that can change system settings has to earn that. Here is how this one is built.</p>
    <div class="grid-3">
      <div class="card"><h3>Read-only by default</h3><p>Nothing is changed until you confirm it. The default state of the product is "looking".</p></div>
      <div class="card"><h3>Restore point first</h3><p>Every system modification creates a restore point before it runs, and can be rolled back from the app.</p></div>
      <div class="card"><h3>Signed updates</h3><p>Updates carry a signature and the installer verifies it before writing. A failed check means no install.</p></div>
      <div class="card"><h3>On-device inference</h3><p>Diagnostics run locally. System data does not leave the machine to be analysed.</p></div>
      <div class="card"><h3>Audit log</h3><p>Every action the product took is recorded, with what it changed and when.</p></div>
      <div class="card"><h3>Two-step verification</h3><p>Available on every account, and required on Business.</p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">DISCLOSURE</span><h2>Found something?</h2></div>
    <div class="card" style="max-width:44em">
      <p>Tell us before you tell anyone else, and we will fix it and credit you unless you would rather we did not.</p>
      <p style="margin-top:14px"><a class="btn btn-primary" href="mailto:security@aipcmaster.com">security@aipcmaster.com</a></p>
    </div>
  </div>
</section>
""",
    ),
    "zh": (
        "安全",
        "AI电脑大师如何保证安全：端侧处理、默认只读、改动前创建还原点、签名更新与完整审计日志。",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">安全</span>
    <h2>它能读你的系统——正因如此，它才格外克制。</h2>
    <p class="lead">一个能改系统设置的工具，必须先证明自己配得上这个权限。下面是它的做法。</p>
    <div class="grid-3">
      <div class="card"><h3>默认只读</h3><p>你不确认，它什么都不改。产品的默认状态就是"在看"。</p></div>
      <div class="card"><h3>先建还原点</h3><p>每一次系统修改之前都会创建还原点，并可在应用内一键回滚。</p></div>
      <div class="card"><h3>签名更新</h3><p>更新包带签名，安装程序在写入之前先校验。校验失败就不安装。</p></div>
      <div class="card"><h3>端侧推理</h3><p>诊断在本地完成。系统数据不会离开这台机器去被分析。</p></div>
      <div class="card"><h3>审计日志</h3><p>产品做过的每一步操作都有记录：改了什么、什么时候。</p></div>
      <div class="card"><h3>两步验证</h3><p>所有账号均可启用，企业版强制要求。</p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">漏洞披露</span><h2>发现了问题？</h2></div>
    <div class="card" style="max-width:44em">
      <p>先告诉我们，再告诉别人。我们会修复，并署名致谢——除非你希望匿名。</p>
      <p style="margin-top:14px"><a class="btn btn-primary" href="mailto:security@aipcmaster.com">security@aipcmaster.com</a></p>
    </div>
  </div>
</section>
""",
    ),
}

# ── docs ───────────────────────────────────────────────────────────────────────

PAGES["docs"] = {
    "en": (
        "Docs",
        "AIPCMaster documentation: getting started, the client guide, diagnostics, optimisation, API reference and enterprise deployment.",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">DOCUMENTATION</span>
    <h2>Everything, in one place</h2>
    <p class="lead">Written for the person using it, not for the person who built it.</p>
    <div class="grid-3">
      <div class="card"><h3>Getting started</h3><p>Install, sign in, run your first diagnostic.</p></div>
      <div class="card"><h3>Client guide</h3><p>Dashboard, device info, reports, subscription and settings.</p></div>
      <div class="card"><h3>Diagnostics</h3><p>What each metric means and how to read a report.</p></div>
      <div class="card"><h3>Optimisation</h3><p>What each action changes, and how to undo it.</p></div>
      <div class="card"><h3>Enterprise deployment</h3><p>Batch install, console, roles and audit export.</p></div>
      <div class="card"><h3>API reference</h3><p>REST endpoints, streaming, auth and error codes.</p></div>
    </div>
  </div>
</section>

<section class="cta">
  <div class="wrap">
    <h2>Not covered here?</h2>
    <p>Ask us directly. A question that needs asking is a page that needs writing.</p>
    <a class="btn btn-primary" href="support.html">Support</a>
    <a class="btn btn-ghost" href="mailto:dev@aipcmaster.com">dev@aipcmaster.com</a>
  </div>
</section>
""",
    ),
    "zh": (
        "文档中心",
        "AI电脑大师文档：快速开始、客户端指南、智能诊断、自动优化、API 参考与企业部署。",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">文档中心</span>
    <h2>所有内容，一处查阅</h2>
    <p class="lead">写给用产品的人，不是写给做产品的人。</p>
    <div class="grid-3">
      <div class="card"><h3>快速开始</h3><p>安装、登录、跑通第一次诊断。</p></div>
      <div class="card"><h3>客户端指南</h3><p>仪表盘、设备信息、报告中心、订阅与设置。</p></div>
      <div class="card"><h3>智能诊断</h3><p>每个指标的含义，以及怎么读懂一份报告。</p></div>
      <div class="card"><h3>自动优化</h3><p>每个动作改了什么，以及怎么撤销。</p></div>
      <div class="card"><h3>企业部署</h3><p>批量安装、控制台、角色与审计导出。</p></div>
      <div class="card"><h3>API 参考</h3><p>REST 端点、流式接口、认证与错误码。</p></div>
    </div>
  </div>
</section>

<section class="cta">
  <div class="wrap">
    <h2>这里没写到的？</h2>
    <p>直接问我们。需要被问的问题，就是需要被写下来的页面。</p>
    <a class="btn btn-primary" href="support.zh.html">支持</a>
    <a class="btn btn-ghost" href="mailto:dev@aipcmaster.com">dev@aipcmaster.com</a>
  </div>
</section>
""",
    ),
}

# ── trial ──────────────────────────────────────────────────────────────────────

PAGES["trial"] = {
    "en": (
        "Trial",
        "Start the AIPCMaster 14-day trial, or ask for a business evaluation. No card required.",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">TRIAL</span>
    <h2>Fourteen days of everything</h2>
    <p class="lead">No card. No sales call. It starts when you sign in and simply ends.</p>
    <div class="grid-3">
      <div class="card"><h3>Personal</h3><p>Install the client and sign in. The trial starts on its own.</p></div>
      <div class="card"><h3>Family</h3><p>Up to five devices on one account, with a shared health report.</p></div>
      <div class="card"><h3>Business</h3><p>A scoped evaluation with a console and a sandbox key.</p></div>
    </div>
    <div class="card" style="max-width:46em;margin-top:26px">
      <h3>Request a trial</h3>
      <p>Tell us which one and how many devices. We reply within one working day.</p>
      <p style="margin-top:16px">
        <a class="btn btn-primary" href="mailto:business@aipcmaster.com?subject=Business%20trial%20request">Business trial</a>
        <a class="btn btn-ghost" href="mailto:support@aipcmaster.com?subject=Personal%20trial">Personal trial</a>
      </p>
    </div>
  </div>
</section>
""",
    ),
    "zh": (
        "试用申请",
        "申请 AI电脑大师 14 天全功能试用，或申请企业评估。无需绑定银行卡。",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">试用申请</span>
    <h2>14 天，全部功能</h2>
    <p class="lead">不用绑卡，不用接销售电话。登录即开始，到期自动结束。</p>
    <div class="grid-3">
      <div class="card"><h3>个人版</h3><p>安装客户端并登录，试用自动开始。</p></div>
      <div class="card"><h3>家庭版</h3><p>一个账号最多 5 台设备，共享设备健康报告。</p></div>
      <div class="card"><h3>企业版</h3><p>带控制台与沙箱密钥的限定范围评估。</p></div>
    </div>
    <div class="card" style="max-width:46em;margin-top:26px">
      <h3>申请试用</h3>
      <p>说明你要哪一种、多少台设备。我们会在一个工作日内回复。</p>
      <p style="margin-top:16px">
        <a class="btn btn-primary" href="mailto:business@aipcmaster.com?subject=%E4%BC%81%E4%B8%9A%E7%89%88%E8%AF%95%E7%94%A8%E7%94%B3%E8%AF%B7">企业版试用</a>
        <a class="btn btn-ghost" href="mailto:support@aipcmaster.com?subject=%E4%B8%AA%E4%BA%BA%E7%89%88%E8%AF%95%E7%94%A8">个人版试用</a>
      </p>
    </div>
  </div>
</section>
""",
    ),
}

# ── support ────────────────────────────────────────────────────────────────────

PAGES["support"] = {
    "en": (
        "Support",
        "AIPCMaster support: email channels, documentation, and answers to the questions that come up most.",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">SUPPORT</span>
    <h2>Get unstuck</h2>
    <p class="lead">Start with the docs. If that does not do it, write to a person.</p>
    <div class="grid-3">
      <div class="card"><h3>Documentation</h3><p>Most answers are already written down.</p><p style="margin-top:14px"><a class="btn btn-ghost" href="docs.html">Open the docs</a></p></div>
      <div class="card"><h3>Email</h3><p>Technical questions and account issues.</p><p style="margin-top:14px"><a class="btn btn-ghost" href="mailto:support@aipcmaster.com">support@aipcmaster.com</a></p></div>
      <div class="card"><h3>Security</h3><p>Report a vulnerability, privately.</p><p style="margin-top:14px"><a class="btn btn-ghost" href="mailto:security@aipcmaster.com">security@aipcmaster.com</a></p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">FAQ</span><h2>Common questions</h2></div>
    <div class="grid-2">
      <div class="card"><h3>Does it slow the machine down?</h3><p>Collection is one sample every five seconds, and inference runs on the NPU or GPU when there is one. On a machine with neither, the light models still run on CPU.</p></div>
      <div class="card"><h3>Does it work offline?</h3><p>Yes. Diagnostics and optimisation are local. Only the account and subscription need the network.</p></div>
      <div class="card"><h3>What does it send to the cloud?</h3><p>Account and subscription data. Not your system metrics — see the privacy page.</p></div>
      <div class="card"><h3>Can I undo an optimisation?</h3><p>Yes. Every change has a restore point behind it and a rollback in the app.</p></div>
    </div>
  </div>
</section>
""",
    ),
    "zh": (
        "支持",
        "AI电脑大师支持：邮件渠道、文档中心，以及最常被问到的问题解答。",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">支持</span>
    <h2>把你卡住的地方解决掉</h2>
    <p class="lead">先看文档。如果还是不行，直接写给具体的人。</p>
    <div class="grid-3">
      <div class="card"><h3>文档中心</h3><p>大多数答案已经写下来了。</p><p style="margin-top:14px"><a class="btn btn-ghost" href="docs.zh.html">打开文档</a></p></div>
      <div class="card"><h3>邮件</h3><p>技术问题与账号问题。</p><p style="margin-top:14px"><a class="btn btn-ghost" href="mailto:support@aipcmaster.com">support@aipcmaster.com</a></p></div>
      <div class="card"><h3>安全</h3><p>私密地报告安全漏洞。</p><p style="margin-top:14px"><a class="btn btn-ghost" href="mailto:security@aipcmaster.com">security@aipcmaster.com</a></p></div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">常见问题</span><h2>常被问到的</h2></div>
    <div class="grid-2">
      <div class="card"><h3>会不会拖慢电脑？</h3><p>采集每 5 秒一次；有 NPU 或 GPU 时推理跑在上面。两者都没有时，轻量模型仍可在 CPU 上跑。</p></div>
      <div class="card"><h3>断网能用吗？</h3><p>能。诊断与优化都在本地，只有账号与订阅需要联网。</p></div>
      <div class="card"><h3>它会往云端发什么？</h3><p>账号与订阅数据，不含你的系统指标——详见隐私政策。</p></div>
      <div class="card"><h3>优化能撤销吗？</h3><p>能。每次改动背后都有还原点，应用内可一键回滚。</p></div>
    </div>
  </div>
</section>
""",
    ),
}

# ── privacy ────────────────────────────────────────────────────────────────────

PAGES["privacy"] = {
    "en": (
        "Privacy",
        "AIPCMaster privacy policy: system metrics stay on the device; the cloud holds account and subscription data. Retention, rights and contact.",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">PRIVACY</span>
    <h2>Your system data stays on your system</h2>
    <p class="lead">Last updated 12 September 2026. This page describes what the product does, not what it is allowed to do.</p>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">WHAT IS COLLECTED</span><h2>Two kinds of data, kept apart</h2></div>
    <div class="grid-2">
      <div class="card">
        <h3>On the device only</h3>
        <p>CPU, memory, disk, network, temperature, process, startup item and driver readings; the diagnostic sessions and reports built from them. These are analysed locally and are not uploaded. They leave the machine only if you export a report yourself.</p>
      </div>
      <div class="card">
        <h3>In the cloud</h3>
        <p>Your account (email or phone, and the credential that proves it is you), the device list with its identifiers, your subscription and order records, and support tickets you open. This is what a login and a licence need, and nothing more.</p>
      </div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">RETENTION</span><h2>How long it is kept</h2></div>
    <table class="tbl">
      <thead><tr><th>Data</th><th>Kept</th></tr></thead>
      <tbody>
        <tr><td>Device metrics and diagnostic reports</td><td>On your device, until you delete them</td></tr>
        <tr><td>Account and device records</td><td>While the account exists</td></tr>
        <tr><td>Orders and invoices</td><td>As long as tax law requires</td></tr>
        <tr><td>Audit logs</td><td>12 months</td></tr>
        <tr><td>Support tickets</td><td>24 months after the last reply</td></tr>
      </tbody>
    </table>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">YOUR RIGHTS</span><h2>What you can ask for</h2></div>
    <div class="grid-4">
      <div class="card"><h3>Export</h3><p>Get a copy of everything held about you.</p></div>
      <div class="card"><h3>Correct</h3><p>Fix what is wrong, from the app or by asking.</p></div>
      <div class="card"><h3>Delete</h3><p>Close the account and have the data removed.</p></div>
      <div class="card"><h3>Object</h3><p>Say no to a processing purpose, and we stop it.</p></div>
    </div>
    <p style="margin-top:26px">Write to <a href="mailto:support@aipcmaster.com" style="color:var(--accent)">support@aipcmaster.com</a>. Security questions go to <a href="mailto:security@aipcmaster.com" style="color:var(--accent)">security@aipcmaster.com</a>.</p>
  </div>
</section>
""",
    ),
    "zh": (
        "隐私政策",
        "AI电脑大师隐私政策：系统指标留在设备本地，云端只保存账号与订阅数据。含保留期限、用户权利与联系方式。",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">隐私政策</span>
    <h2>你的系统数据，留在你的系统里</h2>
    <p class="lead">最后更新：2026 年 9 月 12 日。本页描述产品实际怎么做，而不是它被允许怎么做。</p>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">收集什么</span><h2>两类数据，彼此分开</h2></div>
    <div class="grid-2">
      <div class="card">
        <h3>只在设备本地</h3>
        <p>CPU、内存、磁盘、网络、温度、进程、启动项与驱动的读数，以及由此生成的诊断会话与报告。这些都在本地分析，不上传。只有你主动导出报告时，它们才会离开这台机器。</p>
      </div>
      <div class="card">
        <h3>在云端</h3>
        <p>你的账号（邮箱或手机号，以及证明是你本人的凭据）、带标识的设备列表、订阅与订单记录、以及你提交的工单。登录和授权需要这些，仅此而已。</p>
      </div>
    </div>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">保留期限</span><h2>数据保留多久</h2></div>
    <table class="tbl">
      <thead><tr><th>数据</th><th>保留期限</th></tr></thead>
      <tbody>
        <tr><td>设备指标与诊断报告</td><td>留在你的设备上，直到你删除</td></tr>
        <tr><td>账号与设备记录</td><td>账号存续期间</td></tr>
        <tr><td>订单与发票</td><td>按税法要求保留</td></tr>
        <tr><td>审计日志</td><td>12 个月</td></tr>
        <tr><td>支持工单</td><td>最后一次回复后 24 个月</td></tr>
      </tbody>
    </table>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">你的权利</span><h2>你可以要求什么</h2></div>
    <div class="grid-4">
      <div class="card"><h3>导出</h3><p>获取一份关于你的全部数据副本。</p></div>
      <div class="card"><h3>更正</h3><p>在应用内，或通过联系我们来修正。</p></div>
      <div class="card"><h3>删除</h3><p>注销账号并移除相关数据。</p></div>
      <div class="card"><h3>拒绝</h3><p>对某项处理目的说不，我们就停止。</p></div>
    </div>
    <p style="margin-top:26px">写信到 <a href="mailto:support@aipcmaster.com" style="color:var(--accent)">support@aipcmaster.com</a>；安全相关问题请发 <a href="mailto:security@aipcmaster.com" style="color:var(--accent)">security@aipcmaster.com</a>。</p>
  </div>
</section>
""",
    ),
}

# ── terms ──────────────────────────────────────────────────────────────────────

PAGES["terms"] = {
    "en": (
        "Terms",
        "AIPCMaster terms of service: licence, subscription and renewal, refunds, acceptable use, and the limits of our liability.",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">TERMS</span>
    <h2>Terms of service</h2>
    <p class="lead">Last updated 12 September 2026. Plain language, because a term nobody reads is not a term.</p>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="grid-2">
      <div class="card"><h3>1. The licence</h3><p>We grant you a non-exclusive, non-transferable right to use the software on the number of devices your subscription covers. You may not resell it, or reverse-engineer it beyond what the law allows.</p></div>
      <div class="card"><h3>2. Subscription and renewal</h3><p>Paid plans run for a year from the payment date. Auto-renewal is a switch you control, off by default, and it can be turned off at any time before the renewal date.</p></div>
      <div class="card"><h3>3. Refunds</h3><p>Within 14 days of a charge, ask from the subscription page and we refund it. After that, the remainder of the term is not refundable, except where the law says otherwise.</p></div>
      <div class="card"><h3>4. Acceptable use</h3><p>Do not use the product on machines you are not authorised to administer, and do not use it to interfere with someone else's system.</p></div>
      <div class="card"><h3>5. What it does to your machine</h3><p>Optimisation changes system settings. It asks first, creates a restore point, and can be rolled back. You remain responsible for the changes you approve.</p></div>
      <div class="card"><h3>6. Availability</h3><p>We aim to keep the cloud services running, but do not promise uninterrupted service. Diagnostics work offline regardless.</p></div>
      <div class="card"><h3>7. Liability</h3><p>To the extent the law allows, our liability is limited to the amount you paid in the preceding 12 months.</p></div>
      <div class="card"><h3>8. Changes</h3><p>We may update these terms. Material changes are announced in the app and by email before they take effect.</p></div>
    </div>
    <p style="margin-top:26px">Questions: <a href="mailto:support@aipcmaster.com" style="color:var(--accent)">support@aipcmaster.com</a></p>
  </div>
</section>
""",
    ),
    "zh": (
        "用户协议",
        "AI电脑大师用户协议：授权范围、订阅与续费、退款、使用规范，以及责任范围。",
        """
<section>
  <div class="wrap">
    <span class="sec-tag">用户协议</span>
    <h2>服务条款</h2>
    <p class="lead">最后更新：2026 年 9 月 12 日。用大白话写——没人读的条款不算条款。</p>
  </div>
</section>

<section>
  <div class="wrap">
    <div class="grid-2">
      <div class="card"><h3>1. 授权范围</h3><p>我们授予你在订阅覆盖的设备数量上使用本软件的非独占、不可转让的权利。不得转售，也不得超出法律允许范围进行逆向工程。</p></div>
      <div class="card"><h3>2. 订阅与续费</h3><p>付费方案自付款日起一年有效。自动续费由你控制的开关决定，默认关闭，且可在续费日前随时关闭。</p></div>
      <div class="card"><h3>3. 退款</h3><p>扣费后 14 天内，可在订阅页面直接申请退款。超过该期限，剩余期限不予退款，法律另有规定除外。</p></div>
      <div class="card"><h3>4. 使用规范</h3><p>不得在未获授权管理的机器上使用本产品，也不得用它干扰他人的系统。</p></div>
      <div class="card"><h3>5. 它对系统做了什么</h3><p>优化会修改系统设置。它先征得同意、创建还原点，并可回滚。你确认的改动由你负责。</p></div>
      <div class="card"><h3>6. 可用性</h3><p>我们尽力保持云服务可用，但不承诺不中断。诊断功能离线照常工作。</p></div>
      <div class="card"><h3>7. 责任范围</h3><p>在法律允许的范围内，我们的责任上限为你此前 12 个月已支付的金额。</p></div>
      <div class="card"><h3>8. 条款变更</h3><p>我们可能更新本条款。重大变更会在生效前通过应用内通知与邮件告知。</p></div>
    </div>
    <p style="margin-top:26px">如有疑问：<a href="mailto:support@aipcmaster.com" style="color:var(--accent)">support@aipcmaster.com</a></p>
  </div>
</section>
""",
    ),
}

# ── 404 ────────────────────────────────────────────────────────────────────────

PAGES["404"] = {
    "en": (
        "Not found",
        "That page does not exist.",
        """
<section class="cta" style="padding-top:120px">
  <div class="wrap">
    <span class="sec-tag">404</span>
    <h2>That page does not exist</h2>
    <p>It may have moved, or the link that brought you here may be wrong.</p>
    <a class="btn btn-primary" href="index.html">Back to the homepage</a>
    <a class="btn btn-ghost" href="docs.html">Documentation</a>
  </div>
</section>
""",
    ),
    "zh": (
        "页面不存在",
        "该页面不存在。",
        """
<section class="cta" style="padding-top:120px">
  <div class="wrap">
    <span class="sec-tag">404</span>
    <h2>这个页面不存在</h2>
    <p>它可能被移动过，或者带你来的那个链接有问题。</p>
    <a class="btn btn-primary" href="index.zh.html">回到首页</a>
    <a class="btn btn-ghost" href="docs.zh.html">文档中心</a>
  </div>
</section>
""",
    ),
}
