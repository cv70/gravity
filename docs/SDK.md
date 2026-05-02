# JavaScript 追踪 SDK

## 1. 设计目标

Gravity SDK 用于在网页端采集用户身份、页面浏览、行为事件和转化事件，并把数据稳定送到 Gravity API。它只负责采集、缓存、合并和上报，不承载业务规则。

## 2. 安装

```html
<script src="https://cdn.example.com/gravity-sdk.min.js"></script>
```

或通过 npm：

```bash
npm install @gravity/sdk
```

```javascript
import Gravity from '@gravity/sdk'

const gravity = new Gravity({
  apiKey: 'your_api_key',
  apiHost: 'https://your-domain.com',
  trackRootDomain: '.your-domain.com', // 用于跨子域追踪
})
```

## 3. 初始化配置

```typescript
interface GravityOptions {
  apiKey: string           // 项目 API Key
  apiHost?: string         // API 主机地址，默认 https://app.gravity.com
  trackRootDomain?: string // 跨域追踪的根域名
  debug?: boolean          // 开启调试日志
  enableCookie?: boolean   // 启用 cookie 追踪，默认 true
  cookieExpiry?: number    // cookie 过期天数，默认 365
  encrypt?: boolean        // 是否加密敏感数据，默认 false
}
```

## 4. 核心方法

### 4.1 identify

关联用户身份和属性：

```javascript
// 基本用法
gravity.identify({
  id: 'user_123456',
  email: 'user@example.com',
  name: '张三',
  phone: '13800138000',
  createdAt: '2024-01-15T10:30:00Z',
  plan: 'enterprise',
  company: '示例公司',
})

// 仅更新 traits（不设置用户 ID）
gravity.identify({
  email: 'user@example.com',
  name: '张三',
})
```

### 4.2 track

追踪用户行为事件：

```javascript
// 基本事件
gravity.track('message.opened', {
  campaign_id: 'camp_001',
  message_id: 'msg_123',
  subject: '春季促销',
})

// 页面浏览
gravity.track('page.viewed', {
  name: '/pricing',
  url: 'https://example.com/pricing',
  referrer: 'https://google.com',
})

// 自定义事件
gravity.track('button.clicked', {
  button_id: 'cta_primary',
  button_text: '立即购买',
  location: 'pricing_page',
})
```

### 4.3 trackPage

快捷追踪页面浏览：

```javascript
gravity.trackPage() // 自动获取当前页面信息

// 或手动指定
gravity.trackPage({
  name: '/checkout',
  url: 'https://example.com/checkout',
  referrer: 'https://example.com/cart',
})
```

### 4.4 trackConversion

追踪转化事件：

```javascript
gravity.trackConversion({
  goal_id: 'goal_purchase',
  value: 99.0,
  currency: 'CNY',
  order_id: 'order_12345',
  items: [
    { id: 'prod_001', name: '商品A', price: 49.0, quantity: 2 },
  ],
})
```

## 5. 匿名追踪

在用户未登录时，SDK 自动生成匿名 ID 存储在 cookie 中：

```javascript
// 获取当前匿名 ID
const anonymousId = gravity.getAnonymousId()

// 手动设置匿名 ID
gravity.setAnonymousId('anon_123456')

// 用户登录后，合并匿名行为与正式用户
gravity.identify({
  id: 'user_123',
  email: 'user@example.com',
})
```

## 6. 批量发送

SDK 会缓存事件并批量发送以提高性能：

```javascript
// 默认批量配置
{
  batchSize: 20,        // 达到 20 条触发发送
  flushInterval: 5000,  // 或 5 秒后发送
}

// 手动触发发送
gravity.flush()

// 监听发送完成
gravity.on('flush', (count) => {
  console.log(`发送了 ${count} 条事件`)
})
```

## 7. 广告追踪

### 7.1 UTM 参数

自动解析并存储 UTM 参数：

```javascript
// 访问 https://example.com?utm_source=wechat&utm_medium=article&utm_campaign=spring_sale
gravity.track('page.viewed') // 自动附加 utm_* 属性

// 手动设置 UTM
gravity.utmFromURL()
```

### 7.2 广告落地页转化追踪

```javascript
// 在转化页面调用
gravity.trackConversion({
  goal_id: 'lead_form',
  value: 1,
})

// SDK 会自动记录 referrer 和 UTM 参数
```

## 8. 隐私与合规

### 8.1 禁用追踪

```javascript
// 全局禁用
gravity.optOut()

// 重新启用
gravity.optIn()
```

### 8.2 匿名化 IP

```javascript
const gravity = new Gravity({
  apiKey: 'your_api_key',
  anonymizeIp: true, // 自动截断 IP 最后一位
})
```

### 8.3 敏感字段过滤

```javascript
gravity.addFilter((event) => {
  if (event.properties.password) {
    delete event.properties.password
  }
  return event
})
```

### 8.4 GDPR 合规

```javascript
// 用户请求删除数据
gravity.deleteUser(anonymousId)

// 获取用户数据
gravity.exportUserData(anonymousId, (data) => {
  console.log(data)
  // 发送给用户或监管机构
})
```

## 9. 调试

```javascript
// 开启调试模式
gravity.setDebug(true)

// 查看事件发送日志
gravity.on('track', (event) => {
  console.log('事件:', event.name, event.properties)
})

gravity.on('error', (error) => {
  console.error('SDK 错误:', error)
})
```

## 10. React 集成

```jsx
import { useEffect } from 'react'
import { useGravity } from '@gravity/sdk/react'

function App() {
  const gravity = useGravity()

  useEffect(() => {
    gravity.track('app.loaded')
  }, [])

  const handleCTA = () => {
    gravity.track('cta.clicked', { button: 'get_started' })
  }

  return <button onClick={handleCTA}>开始使用</button>
}
```

## 11. Next.js 集成

```typescript
// lib/gravity.ts
import Gravity from '@gravity/sdk'

export const gravity = new Gravity({
  apiKey: process.env.NEXT_PUBLIC_GRAVITY_API_KEY,
  apiHost: process.env.NEXT_PUBLIC_GRAVITY_API_HOST,
})

// pages/_app.tsx
import { useEffect } from 'react'
import { useRouter } from 'next/router'

export default function App({ Component, pageProps }) {
  const router = useRouter()

  useEffect(() => {
    const handleRouteChange = (url) => {
      gravity.trackPage({ name: url })
    }

    router.events.on('routeChangeComplete', handleRouteChange)
    return () => router.events.off('routeChangeComplete', handleRouteChange)
  }, [])

  return <Component {...pageProps} />
}
```

## 12. API 端点

SDK 发送数据到以下端点：

| 方法 | 端点 | 说明 |
|------|------|------|
| POST | `/api/v1/track/event` | 事件追踪 |
| POST | `/api/v1/track/identify` | 用户识别 |
| POST | `/api/v1/track/page` | 页面浏览 |
| POST | `/api/v1/track/conversion` | 转化追踪 |

## 13. 错误处理

```javascript
gravity.on('error', (error) => {
  if (error.type === 'network') {
    // 网络错误，事件已缓存，稍后重试
  } else if (error.type === 'validation') {
    // 数据验证错误
  }
})
```

## 14. 类型定义

```typescript
interface TrackEvent {
  event: string
  properties?: Record<string, unknown>
  timestamp?: string
  anonymousId?: string
  userId?: string
  context?: {
    ip?: string
    userAgent?: string
    locale?: string
    timezone?: string
    referrer?: string
  }
}

interface IdentifyEvent {
  id?: string
  email?: string
  name?: string
  phone?: string
  traits?: Record<string, unknown>
  anonymousId?: string
}
```
