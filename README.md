# Blog

这是我的个人博客

通过 warp 实现 http 服务，同时通过 askama 实现 markdown 的渲染

主要处理逻辑均集中在后端，前端仅负责 css 与 askama 所需的 template

## 用法

```bash
git clone https://github.com/mofee-11/blog.git
cd blog
cargo r
```

即可访问 `http://localhost:3000`

## 项目文件结构

```bash
$ eza -T -L 3
.
├── Cargo.lock
├── Cargo.toml
├── posts
├── README.md
├── src
│  ├── main.rs
│  ├── md.rs
│  └── web.rs
├── static
│  ├── home.css
│  └── post.css
└── templates
   ├── index.html
   └── post.html 
```

- `post` 放置 md 文件，而且这些文件符合以下特征
  - 文件名是 `20240405223211` 格式
  - 文件内容头部可以有 yaml front matter
  - 没有子文件夹
- `static` 放置前端所需的静态资源
