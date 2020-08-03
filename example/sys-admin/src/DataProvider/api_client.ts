import { stringify } from 'query-string';
import { fetchUtils, DataProvider } from 'ra-core';

/**
 * Maps react-admin queries to a simple REST API
 *
 * This REST dialect is similar to the one of FakeRest
 *
 * @see https://github.com/marmelab/FakeRest
 *
 * @example
 *
 * getList     => GET http://my.api.url/post?sort=['title','ASC']&range=[0, 24]
 * getOne      => GET http://my.api.url/post/123
 * getMany     => GET http://my.api.url/post?filter={id:[123,456,789]}
 * update      => PUT http://my.api.url/post/123
 * create      => POST http://my.api.url/post
 * delete      => DELETE http://my.api.url/post/123
 *
 * @example
 *
 * import * as React from "react";
 * import { Admin, Resource } from 'react-admin';
 * import apiClient from 'api_client';
 *
 * import { PostList } from './post';
 *
 * const App = () => (
 *     <Admin dataProvider={simpleRestProvider('http://path.to.my.api/')}>
 *         <Resource name="post" list={PostList} />
 *     </Admin>
 * );
 *
 * export default App;
 */
export default (apiUrl: string, httpClient = fetchUtils.fetchJson): DataProvider => ({
    getList: (resource, params) => {
        const { page, perPage } = params.pagination;
        const { field, order } = params.sort;
        const query = {
            ...fetchUtils.flattenObject(params.filter),
            sort: field,
            order: order,
            offset: (page - 1) * perPage,
            limit: perPage,
        };
        const url = `${apiUrl}/${resource}?${stringify(query)}`;

        return httpClient(url).then(({ headers, json }) => {
            // if (!headers.has('x-total-count')) {
            //     throw new Error(
            //         'The X-Total-Count header is missing in the HTTP Response. The api_client Data Provider expects responses for lists of resources to contain this header with the total number of results to build the pagination. If you are using CORS, did you declare X-Total-Count in the Access-Control-Expose-Headers header?'
            //     );
            // }
            let total_count = headers.get('x-total-count') || '100';

            let reslut = json.collection.map((e: any) => {
                let result = {
                    id: e._id,
                    ...e
                };
                delete result._id;
                delete result._key;
                return result;
            });
            
            return {
                data: reslut,
                total: parseInt(
                    total_count,
                    10
                ),
            };
        });
    },

    getOne: (resource, params) => {
        let key = String(params.id).split('/').pop();
        return httpClient(`${apiUrl}/${resource}/${key}`).then(({ json }) => {
            let result = {
                id: json._id,
                ...json
            };
            delete result._id;
            delete result._key;
            return {
                data: result,
            };
        });
    },

    getMany: (resource, params) => {
        const query = {
            id: params.ids,
        };
        const url = `${apiUrl}/${resource}?${stringify(query)}`;
        return httpClient(url).then(({ json }) => {
            let result = json.collection.map((e: any) => {
                let res = {
                    id: e._id,
                    ...e
                };
                delete res._id;
                delete res._key;
                return res;
            });
            return { data: result };
        });
    },

    getManyReference: (resource, params) => {
        const { page, perPage } = params.pagination;
        const { field, order } = params.sort;
        const query = {
            ...fetchUtils.flattenObject(params.filter),
            [params.target]: params.id,
            sort: field,
            order: order,
            offset: (page - 1) * perPage,
            limit: perPage,
        };
        const url = `${apiUrl}/${resource}?${stringify(query)}`;

        return httpClient(url).then(({ headers, json }) => {
            // if (!headers.has('x-total-count')) {
            //     throw new Error(
            //         'The X-Total-Count header is missing in the HTTP Response. The api_client Data Provider expects responses for lists of resources to contain this header with the total number of results to build the pagination. If you are using CORS, did you declare X-Total-Count in the Access-Control-Expose-Headers header?'
            //     );
            // }
            let total_count = headers.get('x-total-count') || '100';

            let result = json.collection.map((e: any) => {
                let res = {
                    id: e._id,
                    ...e
                };
                delete res._id;
                delete res._key;
                return res;
            });

            return {
                data: result,
                total: parseInt(
                    total_count,
                    10
                ),
            };
        });
    },

    update: (resource, params) => {
        const key = String(params.id).split('/').pop();
        if (params.data.image.rawFile instanceof File) {
            params.data.image = params.data.image.src
        }
        return httpClient(`${apiUrl}/${resource}/${key}`, {
            method: 'PATCH',
            body: JSON.stringify(params.data),
        }).then(({ json }) => ({ data: json }))
    },

    // api_client doesn't provide an updateMany route, so we fallback to calling update n times instead
    updateMany: (resource, params) =>
        Promise.all(
            params.ids.map(id => {
                const key = String(id).split('/').pop();
                return httpClient(`${apiUrl}/${resource}/${key}`, {
                    method: 'PATCH',
                    body: JSON.stringify(params.data),
                })
            })
        ).then(responses => ({ data: responses.map(({ json }) => json.id) })),

    create: (resource, params) =>
        httpClient(`${apiUrl}/${resource}`, {
            method: 'POST',
            body: JSON.stringify(params.data),
        }).then(({ json }) => ({
            data: { ...params.data, id: json.id },
        })),

    delete: (resource, params) => {
        const key = String(params.id).split('/').pop();
        return httpClient(`${apiUrl}/${resource}/${key}`, {
            method: 'DELETE',
        }).then(({ json }) => ({ data: json }))
    },

    // api_client doesn't handle filters on DELETE route, so we fallback to calling DELETE n times instead
    deleteMany: (resource, params) =>
        Promise.all(
            params.ids.map(id => {
                const key = String(id).split('/').pop();
                return httpClient(`${apiUrl}/${resource}/${key}`, {
                    method: 'DELETE',
                })
            })
        ).then(responses => ({ data: responses.map(({ json }) => json.id) })),
});
