---
title: How to build an agent
author: DaviRain
date: 2025-07-21
tags: agent, go
category: Development
---

# 如何构建一个代理程序
## 或者：皇帝的新衣

**Thorsten Ball，2025年4月15日**

构建一个功能完整的代码编辑代理程序其实并不难。

看起来应该很难。当你看到一个代理程序在编辑文件、运行命令、从错误中脱身、尝试不同策略时——似乎背后一定有什么秘密。

但其实没有。它就是一个LLM、一个循环，以及足够的token。这就是我们在[播客](/podcast)中从一开始就在说的。其余的，那些让Amp如此令人上瘾和印象深刻的东西？都是苦工。

但是构建一个小巧而又令人印象深刻的代理程序甚至不需要那些。你可以用不到400行代码完成，其中大部分还是样板代码。

我现在就要向你展示如何做到这一点。我们将一起编写一些代码，从零行代码开始，到"哇，这是...游戏改变者"。

我强烈建议你跟着做。不，真的。你可能会觉得只是读一遍就够了，不用真的敲代码，但这不到400行代码。我需要你感受一下代码量有多少，我希望你在自己的终端、自己的文件夹里亲眼看到这一切。

我们需要的东西：
- [Go](https://go.dev/)
- [Anthropic API密钥](https://console.anthropic.com/settings/keys)，设置为环境变量`ANTHROPIC_API_KEY`

准备好铅笔！

让我们直接开始，用四个简单命令建立一个新的Go项目：

```bash
mkdir code-editing-agent
cd code-editing-agent
go mod init agent
touch main.go
```

现在，让我们打开`main.go`，作为第一步，放入我们需要的基本框架：

```go
package main

import (
    "bufio"
    "context"
    "fmt"
    "os"
    "github.com/anthropics/anthropic-sdk-go"
)

func main() {
    client := anthropic.NewClient()
    scanner := bufio.NewScanner(os.Stdin)

    getUserMessage := func() (string, bool) {
        if !scanner.Scan() {
            return "", false
        }
        return scanner.Text(), true
    }

    agent := NewAgent(&client, getUserMessage)
    err := agent.Run(context.TODO())
    if err != nil {
        fmt.Printf("Error: %s\n", err.Error())
    }
}

func NewAgent(client *anthropic.Client, getUserMessage func() (string, bool)) *Agent {
    return &Agent{
        client: client,
        getUserMessage: getUserMessage,
    }
}

type Agent struct {
    client *anthropic.Client
    getUserMessage func() (string, bool)
}
```

是的，这还不能编译。但我们这里有的是一个`Agent`，它可以访问`anthropic.Client`（默认情况下会寻找`ANTHROPIC_API_KEY`），并且可以通过从终端的stdin读取来获取用户消息。

现在让我们添加缺失的`Run()`方法：

```go
// main.go
func (a *Agent) Run(ctx context.Context) error {
    conversation := []anthropic.MessageParam{}
    fmt.Println("与Claude聊天（使用'ctrl-c'退出）")

    for {
        fmt.Print("\u001b[94m你\u001b[0m: ")
        userInput, ok := a.getUserMessage()
        if !ok {
            break
        }

        userMessage := anthropic.NewUserMessage(anthropic.NewTextBlock(userInput))
        conversation = append(conversation, userMessage)

        message, err := a.runInference(ctx, conversation)
        if err != nil {
            return err
        }

        conversation = append(conversation, message.ToParam())

        for _, content := range message.Content {
            switch content.Type {
            case "text":
                fmt.Printf("\u001b[93mClaude\u001b[0m: %s\n", content.Text)
            }
        }
    }

    return nil
}

func (a *Agent) runInference(ctx context.Context, conversation []anthropic.MessageParam) (*anthropic.Message, error) {
    message, err := a.client.Messages.New(ctx, anthropic.MessageNewParams{
        Model: anthropic.ModelClaude3_7SonnetLatest,
        MaxTokens: int64(1024),
        Messages: conversation,
    })
    return message, err
}
```

代码不多，对吧？90行，其中最重要的是`Run()`中的这个循环，让我们可以与Claude对话。但这已经是这个程序的心跳了。

对于一个心跳来说，它非常直接：我们首先打印一个提示，要求用户输入内容，将其添加到对话中，发送给Claude，将Claude的响应添加到对话中，打印响应，然后循环继续。

这就是你使用过的每一个AI聊天应用程序，只不过是在终端中。

让我们运行它：

```bash
export ANTHROPIC_API_KEY="这是我最后一次告诉你要设置这个"
# 下载依赖
go mod tidy
# 运行
go run main.go
```

然后你就可以和Claude聊天了，像这样：

注意我们如何在多轮对话中保持同一个对话。它记住了我在第一条消息中的名字。`conversation`在每一轮中都会变长，我们每次都发送整个对话。服务器——Anthropic的服务器——是无状态的。它只能看到`conversation`切片中的内容。维护状态是我们的责任。

好的，让我们继续，因为昵称很糟糕，而且这还不是一个代理程序。什么是代理程序？这是[我的定义](https://youtu.be/J1-W9O3n7j8?t=72)：一个可以访问工具的LLM，让它具有修改上下文窗口之外内容的能力。

## 第一个工具

具有工具访问权限的LLM？什么是工具？基本思想是这样的：你向模型发送一个提示，说如果它想使用"工具"，应该以某种特定方式回复。然后你，作为消息的接收者，通过执行工具来"使用工具"并回复结果。就是这样。我们将看到的其他一切都只是在此基础上的抽象。

想象你正在和朋友交谈，你告诉他们："在接下来的对话中，如果你想让我举起手臂，就眨眼睛"。说出来很奇怪，但这是一个容易理解的概念。

我们已经可以在不改变任何代码的情况下尝试它了。

我们告诉Claude，当它想了解天气时，用`get_weather`眨眼。下一步是举起我们的手臂并回复"工具的结果"：

这在第一次尝试时就运行得很好，不是吗？

这些模型经过训练和微调来使用"工具"，它们非常渴望这样做。到2025年，它们基本上"知道"自己不是无所不知的，可以使用工具来获取更多信息。（当然这并不是实际发生的情况，但现在这是一个足够好的解释。）

总结一下，工具和工具使用只有两个要素：
1. 你告诉模型有哪些工具可用
2. 当模型想要执行工具时，它告诉你，你执行工具并发送响应

为了使(1)更容易，大型模型提供商构建了内置API来发送工具定义。

好的，现在让我们构建我们的第一个工具：`read_file`

## `read_file`工具

为了定义`read_file`工具，我们将使用Anthropic SDK建议的类型，但请记住：在底层，这都将作为字符串发送给模型。这都是"如果你想让我使用`read_file`就眨眼"。

我们要添加的每个工具都需要以下内容：
- 一个名称
- 一个描述，告诉模型工具的作用、何时使用、何时不使用、返回什么等等
- 一个输入模式，作为JSON模式描述，说明这个工具期望什么输入以及以什么形式
- 一个实际执行工具的函数，使用模型发送给我们的输入并返回结果

所以让我们将其添加到我们的代码中：

```go
// main.go
type ToolDefinition struct {
    Name string `json:"name"`
    Description string `json:"description"`
    InputSchema anthropic.ToolInputSchemaParam `json:"input_schema"`
    Function func(input json.RawMessage) (string, error)
}
```

现在我们给我们的`Agent`工具定义：

```go
// main.go
// `tools`在这里添加：
type Agent struct {
    client *anthropic.Client
    getUserMessage func() (string, bool)
    tools []ToolDefinition
}

// 在这里：
func NewAgent(
    client *anthropic.Client,
    getUserMessage func() (string, bool),
    tools []ToolDefinition,
) *Agent {
    return &Agent{
        client: client,
        getUserMessage: getUserMessage,
        tools: tools,
    }
}

// 在这里：
func main() {
    // [... 之前的代码 ...]
    tools := []ToolDefinition{}
    agent := NewAgent(&client, getUserMessage, tools)
    // [... 之前的代码 ...]
}
```

并在`runInference`中将它们发送给模型：

```go
// main.go
func (a *Agent) runInference(ctx context.Context, conversation []anthropic.MessageParam) (*anthropic.Message, error) {
    anthropicTools := []anthropic.ToolUnionParam{}
    for _, tool := range a.tools {
        anthropicTools = append(anthropicTools, anthropic.ToolUnionParam{
            OfTool: &anthropic.ToolParam{
                Name: tool.Name,
                Description: anthropic.String(tool.Description),
                InputSchema: tool.InputSchema,
            },
        })
    }

    message, err := a.client.Messages.New(ctx, anthropic.MessageNewParams{
        Model: anthropic.ModelClaude3_7SonnetLatest,
        MaxTokens: int64(1024),
        Messages: conversation,
        Tools: anthropicTools,
    })
    return message, err
}
```

这里有一些类型操作，我在Go泛型方面还不够熟练，所以我不会试图向你解释`anthropic.String`和`ToolUnionParam`。但是，真的，我发誓，这很简单：

我们发送我们的工具定义，在服务器上，Anthropic然后将这些定义包装在[这个系统提示](https://docs.anthropic.com/en/docs/build-with-claude/tool-use/overview#tool-use-system-prompt)中（内容不多），将其添加到我们的`conversation`中，然后如果模型想使用该工具，就会以特定方式回复。

好的，工具定义正在发送，但我们还没有定义工具。让我们来做这件事并定义`read_file`：

```go
// main.go
var ReadFileDefinition = ToolDefinition{
    Name: "read_file",
    Description: "读取给定相对文件路径的内容。当你想看到文件内部的内容时使用这个。不要对目录名使用这个。",
    InputSchema: ReadFileInputSchema,
    Function: ReadFile,
}

type ReadFileInput struct {
    Path string `json:"path" jsonschema_description:"工作目录中文件的相对路径。"`
}

var ReadFileInputSchema = GenerateSchema[ReadFileInput]()

func ReadFile(input json.RawMessage) (string, error) {
    readFileInput := ReadFileInput{}
    err := json.Unmarshal(input, &readFileInput)
    if err != nil {
        panic(err)
    }

    content, err := os.ReadFile(readFileInput.Path)
    if err != nil {
        return "", err
    }

    return string(content), nil
}

func GenerateSchema[T any]() anthropic.ToolInputSchemaParam {
    reflector := jsonschema.Reflector{
        AllowAdditionalProperties: false,
        DoNotReference: true,
    }
    var v T
    schema := reflector.Reflect(v)
    return anthropic.ToolInputSchemaParam{
        Properties: schema.Properties,
    }
}
```

代码不多，对吧？这是一个单一函数`ReadFile`，以及模型将看到的两个描述：我们的`Description`描述工具本身（"读取给定相对文件路径的内容..."），以及这个工具唯一输入参数的描述（"工作目录中文件的相对路径..."）。

`ReadFileInputSchema`和`GenerateSchema`这些东西？我们需要这些，这样我们就可以为我们发送给模型的工具定义生成JSON模式。为此，我们使用`jsonschema`包，我们需要导入和下载：

```go
// main.go
package main

import (
    "bufio"
    "context"
    // 添加这个：
    "encoding/json"
    "fmt"
    "os"
    "github.com/anthropics/anthropic-sdk-go"
    // 添加这个：
    "github.com/invopop/jsonschema"
)
```

然后运行以下命令：

```bash
go mod tidy
```

然后，在`main`函数中，我们需要确保使用定义：

```go
func main() {
    // [... 之前的代码 ...]
    tools := []ToolDefinition{ReadFileDefinition}
    // [... 之前的代码 ...]
}
```

是时候试试了！

等等，什么？哈哈哈，它想使用工具！显然你的输出会略有不同，但Claude肯定听起来知道它可以读取文件，对吧？

问题是我们没有监听！当Claude眨眼时，我们忽略了它。我们需要修复这个问题。

在这里，让我展示如何通过用以下内容替换我们`Agent`的`Run`方法来做到这一点：

```go
// main.go
func (a *Agent) Run(ctx context.Context) error {
    conversation := []anthropic.MessageParam{}
    fmt.Println("与Claude聊天（使用'ctrl-c'退出）")
    readUserInput := true

    for {
        if readUserInput {
            fmt.Print("\u001b[94m你\u001b[0m: ")
            userInput, ok := a.getUserMessage()
            if !ok {
                break
            }
            userMessage := anthropic.NewUserMessage(anthropic.NewTextBlock(userInput))
            conversation = append(conversation, userMessage)
        }

        message, err := a.runInference(ctx, conversation)
        if err != nil {
            return err
        }

        conversation = append(conversation, message.ToParam())

        toolResults := []anthropic.ContentBlockParamUnion{}
        for _, content := range message.Content {
            switch content.Type {
            case "text":
                fmt.Printf("\u001b[93mClaude\u001b[0m: %s\n", content.Text)
            case "tool_use":
                result := a.executeTool(content.ID, content.Name, content.Input)
                toolResults = append(toolResults, result)
            }
        }

        if len(toolResults) == 0 {
            readUserInput = true
            continue
        }

        readUserInput = false
        conversation = append(conversation, anthropic.NewUserMessage(toolResults...))
    }

    return nil
}

func (a *Agent) executeTool(id, name string, input json.RawMessage) anthropic.ContentBlockParamUnion {
    var toolDef ToolDefinition
    var found bool
    for _, tool := range a.tools {
        if tool.Name == name {
            toolDef = tool
            found = true
            break
        }
    }

    if !found {
        return anthropic.NewToolResultBlock(id, "tool not found", true)
    }

    fmt.Printf("\u001b[92mtool\u001b[0m: %s(%s)\n", name, input)
    response, err := toolDef.Function(input)
    if err != nil {
        return anthropic.NewToolResultBlock(id, err.Error(), true)
    }

    return anthropic.NewToolResultBlock(id, response, false)
}
```

眯着眼看，你会发现90%是样板代码，10%是重要的：当我们从Claude那里得到一个`message`时，我们通过查找`content.Type == "tool_use"`来检查Claude是否要求我们执行工具，如果是，我们交给`executeTool`，在我们的本地注册表中按名称查找工具，解码输入，执行它，返回结果。如果是错误，我们翻转一个布尔值。就是这样。

（是的，循环中有循环，但这不重要。）

我们执行工具，将结果发送回Claude，然后再次请求Claude的响应。真的：就是这样。让我展示给你看。

准备工作，运行这个：

```bash
echo 'what animal is the most disagreeable because it always says neigh?' >> secret-file.txt
```

这在我们的目录中创建了一个`secret-file.txt`，包含一个神秘的谜语。

在同一个目录中，让我们运行我们新的使用工具的代理程序，并要求它查看文件：

让我们深呼吸，一起说出来。准备好了吗？开始：太棒了。你只是给它一个工具，它就...在认为有助于解决任务时使用它。记住：我们没有说任何关于"如果用户询问文件，就读取文件"的话。我们也没有说"如果某些东西看起来像文件名，就想办法读取它"。不，这些都没有。我们说"帮我解决这个文件中的问题"，Claude意识到它可以读取文件来回答，然后就去做了。

当然，我们可以具体一点，真正推动它使用工具，但它基本上是自己完成的：

非常准确。好的，现在我们知道如何让Claude使用工具，让我们添加更多工具。

## `list_files`工具

如果你和我一样，当你登录到一台新计算机时，你做的第一件事就是通过运行`ls`来了解情况——列出文件。

让我们给Claude同样的能力，一个列出文件的工具。这是`list_files`工具的完整实现：

```go
// main.go
var ListFilesDefinition = ToolDefinition{
    Name: "list_files",
    Description: "列出给定路径下的文件和目录。如果没有提供路径，列出当前目录中的文件。",
    InputSchema: ListFilesInputSchema,
    Function: ListFiles,
}

type ListFilesInput struct {
    Path string `json:"path,omitempty" jsonschema_description:"可选的相对路径，用于列出文件。如果不提供，默认为当前目录。"`
}

var ListFilesInputSchema = GenerateSchema[ListFilesInput]()

func ListFiles(input json.RawMessage) (string, error) {
    listFilesInput := ListFilesInput{}
    err := json.Unmarshal(input, &listFilesInput)
    if err != nil {
        panic(err)
    }

    dir := "."
    if listFilesInput.Path != "" {
        dir = listFilesInput.Path
    }

    var files []string
    err = filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
        if err != nil {
            return err
        }
        relPath, err := filepath.Rel(dir, path)
        if err != nil {
            return err
        }
        if relPath != "." {
            if info.IsDir() {
                files = append(files, relPath+"/")
            } else {
                files = append(files, relPath)
            }
        }
        return nil
    })

    if err != nil {
        return "", err
    }

    result, err := json.Marshal(files)
    if err != nil {
        return "", err
    }

    return string(result), nil
}
```

这里没有什么花哨的：`list_files`返回当前文件夹中的文件和目录列表。如果这是一个严肃的努力，我们可以（并且可能应该）做一千种优化，但由于我只是想向你展示巫师帽里有什么，这就够了。

需要注意的一点：我们返回字符串列表，并用尾随斜杠表示目录。这不是必需的，这只是我刚刚决定要做的事情。没有固定格式。只要Claude能理解就行，至于它能否理解，你需要通过实验来确定。你也可以在每个目录前面加上"directory: "或返回一个带有两个标题的Markdown文档："directories"和"files"。有很多选择，你选择哪一个取决于Claude最能理解什么、需要多少token、生成和读取的速度如何等等。

在这里，我们只想创建一个小的`list_files`工具，最简单的选择获胜。

当然，我们也需要告诉Claude关于`list_files`：

```go
// main.go
func main() {
    // [... 之前的代码 ...]
    tools := []ToolDefinition{ReadFileDefinition, ListFilesDefinition}
    // [... 之前的代码 ...]
}
```

就是这样。让我们问Claude它能在这个目录中看到什么。

工作了！它可以读取目录。

但这里有一个重点：Claude知道如何组合这些工具。我们只需要以一种激发它的方式提示它：

首先它使用了`list_files`，然后它用我询问的Go相关文件两次调用了`read_file`。

就...就像我们会做的一样，对吧？我的意思是，在这里，如果我问你我们在这个项目中使用什么版本的Go，你会怎么做？这是Claude为我做的：

Claude查看目录，查看`go.mod`，然后给出答案。

我们现在大约有190行代码。让这个数字深入人心。一旦你感受到了，让我们添加另一个工具。

## 让它`edit_file`

我们要添加的最后一个工具是`edit_file`——一个让Claude编辑文件的工具。

"天哪"，你现在在想，"这就是橡胶遇到路面的地方，这就是他从帽子里拉出兔子的地方。"好吧，让我们看看，好吗？

首先，让我们为我们新的`edit_file`工具添加一个定义：

```go
// main.go
var EditFileDefinition = ToolDefinition{
    Name: "edit_file",
    Description: `对文本文件进行编辑。

在给定文件中将'old_str'替换为'new_str'。'old_str'和'new_str'必须彼此不同。
如果用path指定的文件不存在，将创建它。
`,
    InputSchema: EditFileInputSchema,
    Function: EditFile,
}

type EditFileInput struct {
    Path string `json:"path" jsonschema_description:"文件的路径"`
    OldStr string `json:"old_str" jsonschema_description:"要搜索的文本 - 必须完全匹配且只有一个完全匹配"`
    NewStr string `json:"new_str" jsonschema_description:"用来替换old_str的文本"`
}

var EditFileInputSchema = GenerateSchema[EditFileInput]()
```

没错，我又知道你在想什么："字符串替换来编辑文件？"Claude 3.7喜欢替换字符串（通过实验你会发现它们喜欢或不喜欢什么），所以我们将通过告诉Claude它可以通过用新文本替换现有文本来编辑文件来实现`edit_file`。

现在这是Go中`EditFile`函数的实现：

```go
func EditFile(input json.RawMessage) (string, error) {
    editFileInput := EditFileInput{}
    err := json.Unmarshal(input, &editFileInput)
    if err != nil {
        return "", err
    }

    if editFileInput.Path == "" || editFileInput.OldStr == editFileInput.NewStr {
        return "", fmt.Errorf("invalid input parameters")
    }

    content, err := os.ReadFile(editFileInput.Path)
    if err != nil {
        if os.IsNotExist(err) && editFileInput.OldStr == "" {
            return createNewFile(editFileInput.Path, editFileInput.NewStr)
        }
        return "", err
    }

    oldContent := string(content)
    newContent := strings.Replace(oldContent, editFileInput.OldStr, editFileInput.NewStr, -1)

    if oldContent == newContent && editFileInput.OldStr != "" {
        return "", fmt.Errorf("old_str not found in file")
    }

    err = os.WriteFile(editFileInput.Path, []byte(newContent), 0644)
    if err != nil {
        return "", err
    }

    return "OK", nil
}
```

它检查输入参数，读取文件（或者如果存在就创建它），用`NewStr`替换`OldStr`。然后它将内容写回磁盘并返回"OK"。

仍然缺少的是`createNewFile`，这只是一个小的辅助函数，如果这不是Go的话会短70%：

```go
func createNewFile(filePath, content string) (string, error) {
    dir := path.Dir(filePath)
    if dir != "." {
        err := os.MkdirAll(dir, 0755)
        if err != nil {
            return "", fmt.Errorf("failed to create directory: %w", err)
        }
    }

    err := os.WriteFile(filePath, []byte(content), 0644)
    if err != nil {
        return "", fmt.Errorf("failed to create file: %w", err)
    }

    return fmt.Sprintf("Successfully created file %s", filePath), nil
}
```

最后一步：将其添加到我们发送给Claude的工具列表中。

```go
// main.go
func main() {
    // [... 之前的代码 ...]
    tools := []ToolDefinition{ReadFileDefinition, ListFilesDefinition, EditFileDefinition}
    // [... 之前的代码 ...]
}
```

然后...我们准备好了，但你准备好了吗？你准备好释放它了吗？

我想是的，让我们这样做。让我们告诉Claude用JavaScript创建一个新的FizzBuzz函数。

对吧？！这很令人印象深刻，不是吗？这是你可能想出的`edit_file`——通常是代理程序——最基本的实现。

但是，它有效吗？是的，它有效：

```bash
$ node fizzbuzz.js
Running FizzBuzz:
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
16
[...]
```

太棒了。但是嘿，让它实际编辑一个文件而不只是创建一个。

当我要求"请编辑fizzbuzz.js，使其只打印到15"时，Claude的做法如下：

它读取文件，编辑文件以更改运行时长，然后还编辑文件以更新顶部的注释。

它仍然有效：

```bash
$ node fizzbuzz.js
Running FizzBuzz:
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
```

好的，让我们再做一个，让它做以下事情：

创建一个congrats.js脚本，rot13解码以下字符串'Pbatenghyngvbaf ba ohvyqvat n pbqr-rqvgvat ntrag!'并打印它

也许是一个很高的要求。让我们看看：

它有效吗？让我们试试：

```bash
$ node congrats.js
Congratulations on building a code-editing agent!
```

它有效！

这不是很神奇吗？

如果你和我在过去几个月中交谈过的所有工程师一样，在阅读这篇文章时，你很可能一直在等待兔子从帽子里被拉出来，等待我说"好吧，实际上这比这难得多得多。"但它不是。

这本质上就是代码编辑代理程序内部循环的全部内容。当然，将其集成到你的编辑器中，调整系统提示，在适当的时间给它适当的反馈，围绕它的漂亮UI，围绕工具的更好工具，对多个代理程序的支持等等——我们在Amp中构建了所有这些，但它不需要天才的时刻。所需要的只是实用的工程和苦工。

这些模型现在非常强大。300行代码和三个工具，现在你可以与编辑你代码的外星智能对话。如果你认为"好吧，但我们没有真正..."——去试试吧！去看看你能用这个走多远。我打赌比你想象的要远得多。

这就是为什么我们认为一切都在改变。
