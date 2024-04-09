import requests, time, csv, sys

gh_token = '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'

def api_call(url):
    res = requests.get(url,
        auth=('shinmao', 'git988358@96'),
        headers={
            "Accept": "application/vnd.github.mercy-preview+json",
            "Authorization": "token " + gh_token
        })
    if (res.status_code != 200) and (res.status_code != 422):
        print("status code %s: %s, url: %s" % (res.status_code, res.reason, url))
        time.sleep(300)
        return api_call(url)
    return res.json()

if __name__ == '__main__':
    star_dict = {}
    crate_list = list()
    option = sys.argv[1]
    if option == "req_github":
        pagenum = 10
        for pn in range(1, pagenum + 1):
            # can only show 1000 results per time; therefore, each time we only request for 10 pages
            # each time, change the stars:<number manually
            url = 'https://api.github.com/search/repositories?' + 'q=stars:<342+is:public+mirror:false+lang:Rust'
            url += '&sort=stars&per_page=100' + '&order=desc' + '&page=' + str(pn)
            res = api_call(url)
            if 'items' not in res:
                break
            repolist = res['items']
            with open('star_sorted_crate.txt', 'a+') as output:
                for repo in repolist:
                    repo_url = repo['url']
                    repo_star = repo['stargazers_count']
                    output.write(repo_url + ',' + str(repo_star) + '\n')
                    print(repo_url)
                    print(repo_star)
            output.close()
    elif option == "sort_github":
        gh_dict = {}
        with open('star_sorted_crate.txt', 'r') as gh_list:
            for crate in gh_list:
                splitted = crate.rstrip().split(',')
                url_splitted = splitted[0].split('/')
                crate = url_splitted[-2] + '/' + url_splitted[-1]
                star = splitted[1]
                gh_dict[crate] = star
        gh_list.close()
        with open('crates_listII.txt', 'r') as crate_list:
            for crate in crate_list:
                splitted = crate.rstrip().split(',')
                crate = splitted[0] + '-' + splitted[1]
                # format: user/repo
                url = splitted[2]
                if url == 'none':
                    continue
                url_splitted = url.split('/')
                if len(url_splitted) != 2:
                    continue
                print(url_splitted)
                user_repo = url_splitted[-2] + '/' + url_splitted[-1]
                if user_repo in gh_dict:
                    star_dict[crate] = int(gh_dict[user_repo])
        crate_list.close()
        sorted_crate_list = sorted(star_dict.items(), key=lambda x:x[1], reverse=True)
        sorted_crate_dict = dict(sorted_crate_list)
        w = csv.writer(open('star_crate_list.csv', 'w+'))
        for key, val in sorted_crate_dict.items():
            w.writerow([key, val])