const { execSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const zlib = require('node:zlib');

// 定义关键路径
const CURRENT_DIR = __dirname;
const CARGO_PATH = path.join(CURRENT_DIR, 'Cargo.toml');
const PAGES_DIR = path.join(CURRENT_DIR, 'pages');
const ZIP_PATH = path.join(CURRENT_DIR, 'pages.zip');

console.log("当前目录:", CURRENT_DIR);
console.log("pages 目录:", PAGES_DIR);
console.log("zip 文件路径:", ZIP_PATH);


async function main() {
    try {
        // ==========================================
        // 第一步：运行 cargo build --release 编译
        // ==========================================
        console.log('🦀 [1/3] 开始编译 Rust 项目...');
        // stdio: 'inherit' 可以让 cargo 的编译进度条和错误直接打印在当前终端
        execSync(`cargo build -r --manifest-path ${CARGO_PATH}`, {
            stdio: 'inherit',
            cwd: __dirname,
        });

        // ==========================================
        // 第二步：识别并复制产物，重命名为 pages.node
        // ==========================================
        console.log('📂 [2/3] 正在识别编译产物并复制...');

        // 确保 pages 目录存在
        if (!fs.existsSync(PAGES_DIR)) {
            fs.mkdirSync(PAGES_DIR, { recursive: true });
        }

        // 自动探测当前操作系统，定位 target/release 下的动态库文件名
        const platform = process.platform;
        let sourceLibName = '';

        if (platform === 'win32') {
            // Windows 平台产物通常在根目录或特定子目录下，这里以标准的本地构建为例
            // 如果是 cd 到了其他目录，请根据实际的 Cargo.toml 位置调整相对路径
            sourceLibName = 'pages.dll'; // 👈 请修改为你在 Cargo.toml 中定义的 lib.name
        } else if (platform === 'linux') {
            sourceLibName = 'libpages.so'; // 👈 请修改为你在 Cargo.toml 中定义的 lib.name（加 lib 前缀）
        } else if (platform === 'darwin') {
            sourceLibName = 'libpages.dylib'; // macOS 支持
        } else {
            throw new Error(`暂不支持的操作系统平台: ${platform}`);
        }

        // 寻找 target 目录（优先在当前目录找，找不到往上级找，适应不同项目结构）
        let targetDir = path.join(CURRENT_DIR, 'target', 'release', sourceLibName);
        if (!fs.existsSync(targetDir)) {
            // 尝试往上走一级寻找（针对子目录结构）
            targetDir = path.join(CURRENT_DIR, '..', 'target', 'release', sourceLibName);
        }

        if (!fs.existsSync(targetDir)) {
            throw new Error(`找不到预期的编译产物: ${targetDir}\n请检查 ` + sourceLibName + ` 是否与你的 Cargo.toml 中的 [lib] name 匹配。`);
        }

        const targetNodePath = path.join(PAGES_DIR, 'pages.node');
        fs.copyFileSync(targetDir, targetNodePath);
        console.log(`✨ 成功将 ${sourceLibName} 复制并重命名为 ${targetNodePath}`);


        // ==========================================
        // 第三步：将 pages 目录下的所有文件打包为 pages.zip
        // ==========================================
        console.log('🤐 [3/3] 开始打包 pages 目录至 pages.zip...');

        // 使用 Node.js 自带的 zlib 纯 JS 跨平台打包（零依赖外部 zip 命令）
        await zipDirectory(PAGES_DIR, ZIP_PATH);

        console.log(`🎉 构建成功！产物已生成: ${ZIP_PATH}`);

    } catch (error) {
        console.error('\n❌ 构建失败:', error.message);
        process.exit(1);
    }
}

/**
 * 🚀 纯 JavaScript 实现的 零依赖、全平台通用、支持【无限层级递归】的 Zip 打包函数
 */
function zipDirectory(sourceDir, outZipPath) {
    return new Promise((resolve, reject) => {
        try {
            const zipBuffers = [];
            let centralDirectoryBuffers = [];
            let offset = 0;
            let fileCount = 0;

            // 1. 🛠️ 定义一个内部递归函数，用来深度遍历收集所有文件
            function getAllFiles(dirPath, relativePath = '') {
                const items = fs.readdirSync(dirPath);

                for (const item of items) {
                    const fullPath = path.join(dirPath, item);
                    // 核心：ZIP 内部的路径分隔符必须统一为正斜杠 '/'，即使在 Windows 上也是如此
                    const zipEntryPath = relativePath ? `${relativePath}/${item}` : item;
                    const stat = fs.statSync(fullPath);

                    if (stat.isDirectory()) {
                        // 如果是文件夹，递归进去继续找
                        getAllFiles(fullPath, zipEntryPath);
                    } else {
                        // 如果是文件，开始进行二进制打包处理
                        fileCount++;
                        const fileContent = fs.readFileSync(fullPath);
                        const filenameBuffer = Buffer.from(zipEntryPath, 'utf-8');

                        // 使用 zlib 的 deflateRawSync 进行标准 ZIP 核心数据压缩
                        const compressedContent = zlib.deflateRawSync(fileContent, { level: 9 });
                        const crc = crc32(fileContent);

                        // A. 构建 Local File Header (局部文件头)
                        const lfh = Buffer.alloc(30);
                        lfh.writeUInt32LE(0x04034b50, 0);         // 魔数
                        lfh.writeUInt16LE(20, 4);                  // 所需版本
                        lfh.writeUInt16LE(0, 6);                   // 标志
                        lfh.writeUInt16LE(8, 8);                   // 压缩方法 (8 = Deflate)
                        lfh.writeUInt16LE(0, 10);                  // 修改时间
                        lfh.writeUInt16LE(0, 12);                  // 修改日期
                        lfh.writeUInt32LE(crc, 14);                // CRC-32
                        lfh.writeUInt32LE(compressedContent.length, 18); // 压缩后大小
                        lfh.writeUInt32LE(fileContent.length, 22);       // 原始大小
                        lfh.writeUInt16LE(filenameBuffer.length, 26);    // 相对路径文件名长度
                        lfh.writeUInt16LE(0, 28);                  // 扩展长度

                        zipBuffers.push(lfh, filenameBuffer, compressedContent);

                        // B. 构建 Central Directory File Header (中央目录文件头)
                        const cdfh = Buffer.alloc(46);
                        cdfh.writeUInt32LE(0x02014b50, 0);         // 魔数
                        cdfh.writeUInt16LE(20, 4);                  // 制作版本
                        cdfh.writeUInt16LE(20, 6);                  // 所需版本
                        cdfh.writeUInt16LE(0, 8);                   // 标志
                        cdfh.writeUInt16LE(8, 10);                  // 压缩方法
                        cdfh.writeUInt16LE(0, 12);                  // 修改时间
                        cdfh.writeUInt16LE(0, 14);                  // 修改日期
                        cdfh.writeUInt32LE(crc, 16);                // CRC-32
                        cdfh.writeUInt32LE(compressedContent.length, 20); // 压缩大小
                        cdfh.writeUInt32LE(fileContent.length, 24);       // 原始大小
                        let nLen = filenameBuffer.length;
                        cdfh.writeUInt16LE(nLen, 28);               // 相对路径文件名长度
                        cdfh.writeUInt16LE(0, 30);                  // 扩展长度
                        cdfh.writeUInt16LE(0, 32);                  // 注释长度
                        cdfh.writeUInt16LE(0, 34);                  // 磁盘开始号
                        cdfh.writeUInt16LE(0, 36);                  // 内部属性
                        cdfh.writeUInt32LE(0, 38);                  // 外部属性
                        cdfh.writeUInt32LE(offset, 42);             // 局部文件头相对位移

                        centralDirectoryBuffers.push(cdfh, filenameBuffer);

                        // 计算并更新下一个文件的位移量
                        offset += 30 + filenameBuffer.length + compressedContent.length;
                    }
                }
            }

            // 2. 🚀 启动深度优先递归遍历
            getAllFiles(sourceDir);

            if (fileCount === 0) {
                throw new Error("pages 目录下没有任何可打包的文件！");
            }

            const centralDirOffset = offset;
            const centralDirSize = Buffer.concat(centralDirectoryBuffers).length;

            // 3. 构建 End of Central Directory Record (中央目录结束记录)
            const eocd = Buffer.alloc(22);
            eocd.writeUInt32LE(0x06054b50, 0);         // 魔数
            eocd.writeUInt16LE(0, 4);                  // 当前磁盘号
            eocd.writeUInt16LE(0, 6);                  // 中央目录开始磁盘号
            eocd.writeUInt16LE(fileCount, 8);          // 本磁盘中央目录记录总数
            eocd.writeUInt16LE(fileCount, 10);         // 中央目录记录总数
            eocd.writeUInt32LE(centralDirSize, 12);    // 中央目录大小
            eocd.writeUInt32LE(centralDirOffset, 16);  // 中央目录位移量
            eocd.writeUInt16LE(0, 20);                  // 注释长度

            // 4. 组装并强行一次性写入落盘
            const finalZipBuffer = Buffer.concat([...zipBuffers, ...centralDirectoryBuffers, eocd]);
            fs.writeFileSync(outZipPath, finalZipBuffer);

            resolve();
        } catch (err) {
            reject(err);
        }
    });
}

/**
 * 标准 CRC32 校验函数（保持不变）
 */
function crc32(buffer) {
    const table = new Int32Array(256);
    for (let i = 0; i < 256; i++) {
        let c = i;
        for (let j = 0; j < 8; j++) {
            c = (c & 1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1);
        }
        table[i] = c;
    }
    let crc = -1;
    for (let i = 0; i < buffer.length; i++) {
        crc = (crc >>> 8) ^ table[(crc ^ buffer[i]) & 0xFF];
    }
    return (crc ^ -1) >>> 0;
}

/**
//  * 纯原生 Node.js 实现的文件夹流式打包 Zip 函数
//  */
// function zipDirectory(sourceDir, outZipPath) {
//     return new Promise((resolve, reject) => {
//         // 创建 zip 压缩流
//         const zip = zlib.createGzip({ level: 9 }); // 最高压缩率
//         const output = fs.createWriteStream(outZipPath);

//         // 简单的打包实现（将目录下的文件序列化写入，适合轻量 pages 文件夹）
//         // 注：此处使用更健壮的打包逻辑，遍历目录
//         const files = fs.readdirSync(sourceDir);

//         output.on('close', () => resolve());
//         output.on('error', (err) => reject(err));

//         // 现代 Node.js 可以直接利用内置的打包机制，这里我们采用标准的原生轻量打包
//         // 如果文件结构非常复杂，推荐安装 archiver，但为了满足您“零依赖”的需求，我们使用简单流拼接
//         // 这里采用最稳妥的跨平台兼容压缩，将 pages 目录作为整体文件写入

//         // 由于原生 zlib 包裹多文件较复杂，这里使用原生的底层方法或者调用系统命令作为保底：
//         // try {
//         //     if (process.platform === 'win32') {
//         //         // Windows PowerShell 压缩命令（无需安装任何外置软件）
//         //         execSync(`powershell -Command "Compress-Archive -Path '${sourceDir}\\*' -DestinationPath '${outZipPath}' -Force"`);
//         //     } else {
//         //         // Linux / macOS 自带 zip 命令
//         //         execSync(`zip -r -j "${outZipPath}" "${sourceDir}"/*`);
//         //     }
//         //     resolve();
//         // } catch (e) {
//         //     reject(new Error('系统自带打包命令执行失败，请确保环境具备打包权限。' + e.message));
//         // }
//     });
// }

main();